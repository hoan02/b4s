//! Headphone "789C" framing used by newer Baseus models (e.g. BP1 Ultra).
//!
//! From official app 2.14.1:
//! - `DeviceManager.N0(model)` → wrap with HeadPhoneDataResolveManager.e()
//! - Frame: `789C` + len(u16 BE) + body + CRC16 (custom table)
//! - BP1 Pro does NOT use this wrap; BP1 Ultra does.

use super::crc_table::CRC_TABLE;

/// Models that need 789C+CRC wrap (DeviceManager.N0 == true in app 2.14.1).
pub fn needs_v2_wrap(model_id: Option<&str>, model_name: Option<&str>, ble_name: Option<&str>) -> bool {
    let blob = format!(
        "{} {} {}",
        model_id.unwrap_or(""),
        model_name.unwrap_or(""),
        ble_name.unwrap_or("")
    )
    .to_lowercase();
    // Explicit Ultra / NC / M4s / Inspire family that use N0 framing
    [
        "bp1 ultra",
        "ep10 ultra",
        "bp1 nc",
        "ep10 nc",
        "m4s",
        "inspire",
        "bc1 lite",
        "bc2",
        "bf1 lite",
        "wm01s",
        "wm02s",
        "as01 air",
        "mc2 nc",
        "mc2",
        "mp1",
        "ms1",
    ]
    .iter()
    .any(|k| blob.contains(k))
}

/// Custom CRC16 from HeadPhoneCrcUtil (app 2.14.1).
pub fn crc16(data: &[u8]) -> u16 {
    let mut i2: u32 = 65535;
    for &b in data {
        let i4 = CRC_TABLE[((b as u32) ^ i2) as usize & 0xFF] as u32;
        i2 = ((i2 ^ i4) >> 8).wrapping_add((i4 & 0xFF) << 8);
    }
    (i2 & 0xFFFF) as u16
}

/// Opcode → type byte used in wrap (HeadPhoneDataResolveManager.b).
/// Returns "00" | "01" | "03" as u8.
fn opcode_type(opcode: u8) -> u8 {
    match opcode {
        // type 03 — set / action
        0x10 | 0x22 | 0x24 | 0x26 | 0x2B | 0x2D | 0x2E | 0x34 | 0x37 | 0x3A | 0x3C
        | 0x4A | 0x4C | 0x4F | 0x50 | 0x52 | 0x54 | 0x56 | 0x58 | 0x8D | 0x8F | 0x92
        | 0x94 | 0x97 | 0x9B | 0xA2 | 0xA4 | 0xFE | 0x31 => 0x03,
        // type 01 — query / state (fall-through group in app)
        0x01 | 0x02 | 0x12 | 0x19 | 0x1A | 0x1C | 0x21 | 0x23 | 0x25 | 0x27 | 0x2A
        | 0x2C | 0x2F | 0x30 | 0x33 | 0x35 | 0x36 | 0x38 | 0x40 | 0x4B | 0x4E | 0x51
        | 0x53 | 0x55 | 0x57 | 0x59 | 0x5A | 0x5B | 0x80 | 0x8A | 0x8B | 0x8C | 0x8E
        | 0x91 | 0x93 | 0x96 | 0x9A | 0xA1 | 0xA3 => 0x01,
        // type 00 — simple body
        _ => 0x00,
    }
}

/// Wrap a bare BA command (e.g. [0xBA, 0x34, 0x01, 0x68]) into 789C frame + CRC.
pub fn wrap_ba_command(bare: &[u8]) -> Option<Vec<u8>> {
    if bare.len() < 2 || bare[0] != 0xBA {
        return None;
    }
    let opcode = bare[1];
    let ty = opcode_type(opcode);
    // substring2 = opcode + payload (no BA magic) as hex body bytes
    let mut substring2 = bare[1..].to_vec();

    let mut body = Vec::new();
    body.extend_from_slice(&[0x78, 0x9C]);

    if ty == 0x00 {
        // 789C + len(sub+7) + 02 + substring2
        let len = (substring2.len() + 7) as u16;
        body.extend_from_slice(&len.to_be_bytes());
        body.push(0x02);
        body.extend_from_slice(&substring2);
    } else {
        // special: opcode 0x01 payload forced to FF (legacy)
        if opcode == 0x01 {
            substring2 = vec![0xFF];
        }
        let length = substring2.len() as u16;
        let len_field = length + 9;
        body.extend_from_slice(&len_field.to_be_bytes());
        body.push(0x02);
        body.push(ty);
        body.push(length as u8);
        body.extend_from_slice(&substring2);
    }

    let c = crc16(&body);
    body.push((c >> 8) as u8);
    body.push((c & 0xFF) as u8);
    Some(body)
}

/// The canonical BP1 Ultra battery polling frame.
///
/// Ultra firmware returns earbud and case battery from the wrapped BA02
/// exchange. Do not mix bare BA02 or BA27 frames into this exchange.
pub fn battery_query_frame() -> Vec<u8> {
    let bare = vec![0xBA, 0x02];
    wrap_ba_command(&bare).expect("BA02 always produces a BP1 Ultra frame")
}

/// Extract AA frames from a notify buffer.
///
/// Matches official `HeadPhoneDataResolveManager.getReplyData` (app 2.14.1):
/// - Bare `AA…` → pass through
/// - `789C` frames: CRC-check, then either
///   - subcontracting opcodes `1B`/`4D`: `AA` + bytes[5..end-2]
///   - else walk length-prefixed chunks starting at offset 5:
///     `[type?][len][payload…]` → emit `AA` + payload (payload already starts with opcode)
///
/// Battery L/R is always `AA 02 LL 00 RR 01` after unwrap
/// (`BleUtils.d`: regex `AA02([0-9A-F]{2})00([0-9A-F]{2})01` → `"L-R"`).
pub fn unwrap_notify(data: &[u8]) -> Vec<Vec<u8>> {
    if data.len() >= 2 && data[0] == 0xAA {
        return vec![data.to_vec()];
    }

    let mut out = Vec::new();

    // 789C framed packets (length at [2..4] BE) — official HeadPhoneDataResolveManager.c
    if data.len() >= 6 && data[0] == 0x78 && data[1] == 0x9C {
        let mut i = 0usize;
        while i + 6 <= data.len() {
            let total = u16::from_be_bytes([data[i + 2], data[i + 3]]) as usize;
            if total < 6 || i + total > data.len() {
                break;
            }
            let pkt = &data[i..i + total];
            let body = &pkt[..total - 2];
            let crc_rx = u16::from_be_bytes([pkt[total - 2], pkt[total - 1]]);
            if crc16(body) == crc_rx && total > 7 {
                extract_789c_inner(pkt, &mut out);
            } else if total > 7 {
                // CRC mismatch — still try to salvage AA02/AA27 / length-prefix parse
                log::debug!(
                    "789C CRC mismatch (rx={crc_rx:04X} calc={:04X}), salvage parse",
                    crc16(body)
                );
                extract_789c_inner(pkt, &mut out);
            }
            i += total;
        }
    }

    // Scan for full official battery frames only (partial AA02 is mode-ACK noise)
    // BleUtils.d: AA02 LL 00 RR 01  (exactly 6 bytes)
    let mut i = 0usize;
    while i + 5 < data.len() {
        if data[i] == 0xAA
            && data[i + 1] == 0x02
            && data[i + 3] == 0x00
            && data[i + 5] == 0x01
        {
            push_unique_aa(&mut out, data[i..i + 6].to_vec());
            i += 6;
        } else if data[i] == 0xAA && data[i + 1] == 0x27 {
            let end = (i + 4).min(data.len());
            push_unique_aa(&mut out, data[i..end].to_vec());
            i = end;
        } else {
            i += 1;
        }
    }

    if out.is_empty() && !data.is_empty() {
        out.push(data.to_vec());
    }
    out
}

/// Official 789C inner extract (one packet, includes header+CRC).
fn extract_789c_inner(pkt: &[u8], out: &mut Vec<Vec<u8>>) {
    if pkt.len() < 8 {
        return;
    }
    // Prefer literal AA if present (some firmwares embed it)
    if let Some(pos) = pkt.iter().position(|&b| b == 0xAA) {
        if pos + 2 <= pkt.len().saturating_sub(2) {
            let end = pkt.len() - 2; // strip CRC
            push_unique_aa(out, pkt[pos..end].to_vec());
            // Don't return — also run length-prefix walk; AA may be incomplete
        }
    }

    // bytes[5] = first content code (often type 00/01/03 OR opcode for subcontract)
    let code = pkt[5];
    let end_excl_crc = pkt.len() - 2;

    // Subcontracting opcodes 0x1B / 0x4D → AA + raw from offset 5
    if code == 0x1B || code == 0x4D {
        let mut frame = vec![0xAA];
        frame.extend_from_slice(&pkt[5..end_excl_crc]);
        push_unique_aa(out, frame);
        return;
    }

    // Length-prefixed walk from offset 5 (official else-branch):
    // i6 starts at 5; while i6+1 < end: len=pkt[i6+1], payload=pkt[i6+2..i6+2+len], emit AA+payload
    let mut i6 = 5usize;
    let mut emitted = false;
    while i6 + 1 < end_excl_crc {
        let len = pkt[i6 + 1] as usize;
        let start = i6 + 2;
        let stop = start + len;
        if len == 0 || stop > end_excl_crc {
            break;
        }
        let payload = &pkt[start..stop];
        // payload usually starts with opcode (e.g. 02 64 00 64 01)
        let mut frame = vec![0xAA];
        frame.extend_from_slice(payload);
        push_unique_aa(out, frame);
        emitted = true;
        i6 += len + 1;
    }

    // Fallback: type-byte frames where [4]=0x02, [5]=ty, [6]=len, [7…]=body
    // TX wrap of BA02: 78 9C len 02 01 01 02 CRC — RX similar with longer body
    if !emitted && pkt.len() > 9 && pkt[4] == 0x02 {
        let ty = pkt[5];
        if ty == 0x00 {
            // 789C + len + 02 + substring2  (type 00 path)
            let mut frame = vec![0xAA];
            frame.extend_from_slice(&pkt[5..end_excl_crc]);
            push_unique_aa(out, frame);
        } else if end_excl_crc > 7 {
            let len = pkt[6] as usize;
            if len > 0 && 7 + len <= end_excl_crc {
                let mut frame = vec![0xAA];
                frame.extend_from_slice(&pkt[7..7 + len]);
                push_unique_aa(out, frame);
            }
        }
    }
}

fn push_unique_aa(out: &mut Vec<Vec<u8>>, frame: Vec<u8>) {
    if frame.len() < 2 || frame[0] != 0xAA {
        return;
    }
    if out.iter().any(|f| f == &frame) {
        return;
    }
    // Prefer longer frame when same opcode prefix (avoid truncated AA02)
    if let Some(existing) = out.iter_mut().find(|f| f.len() >= 2 && f[1] == frame[1] && f[0] == 0xAA) {
        if frame.len() > existing.len() {
            *existing = frame;
        }
        return;
    }
    out.push(frame);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wrap_anc() {
        let w = wrap_ba_command(&[0xBA, 0x34, 0x01, 0x68]).unwrap();
        assert_eq!(w[0], 0x78);
        assert_eq!(w[1], 0x9C);
        assert_eq!(&w[0..10], &[0x78, 0x9C, 0x00, 0x0C, 0x02, 0x03, 0x03, 0x34, 0x01, 0x68]);
        assert_eq!(w.len(), 12);
        assert_eq!(&w[10..], &[0xD6, 0xC2]);
    }

    #[test]
    fn wrap_handshake() {
        let w = wrap_ba_command(&[0xBA, 0x05, 0x00]).unwrap();
        assert_eq!(w, vec![0x78, 0x9C, 0x00, 0x09, 0x02, 0x05, 0x00, 0x97, 0x5F]);
    }

    #[test]
    fn wrap_eq() {
        let w = wrap_ba_command(&[0xBA, 0x43, 0x00]).unwrap();
        assert_eq!(w, vec![0x78, 0x9C, 0x00, 0x09, 0x02, 0x43, 0x00, 0xF7, 0x6D]);
    }

    #[test]
    fn wrap_ba02_query() {
        // type 01 (query), substring2 = [0x02], length=1
        // 789C + len(1+9)=0x000A + 02 + 01 + 01 + 02 + CRC
        let w = wrap_ba_command(&[0xBA, 0x02]).unwrap();
        assert_eq!(&w[0..8], &[0x78, 0x9C, 0x00, 0x0A, 0x02, 0x01, 0x01, 0x02]);
        assert_eq!(w.len(), 10);
    }

    #[test]
    fn bp1_ultra_battery_poll_uses_one_wrapped_query() {
        let frame = battery_query_frame();
        assert_eq!(frame[..2], [0x78, 0x9C]);
        assert_eq!(frame[5..8], [0x02, 0x01, 0x01]);
    }

    #[test]
    fn unwrap_bare_aa02() {
        let frames = unwrap_notify(&[0xAA, 0x02, 0x64, 0x00, 0x64, 0x01]);
        assert_eq!(frames.len(), 1);
        assert_eq!(frames[0], vec![0xAA, 0x02, 0x64, 0x00, 0x64, 0x01]);
    }

    #[test]
    fn unwrap_789c_battery_like_official() {
        // Simulate RX: 789C | len | 02 | ty=01 | len=05 | 02 64 00 64 01 | CRC
        // Build with real CRC via wrap path then patch — or craft + crc16
        let mut body = vec![
            0x78, 0x9C, 0x00, 0x0E, // head + total len 14
            0x02, 0x01, 0x05, // fixed 02, type 01, payload len 5
            0x02, 0x64, 0x00, 0x64, 0x01, // battery body (opcode+LL+00+RR+01)
        ];
        // total field should equal body.len()+2
        let total = (body.len() + 2) as u16;
        body[2] = (total >> 8) as u8;
        body[3] = (total & 0xFF) as u8;
        let c = crc16(&body);
        body.push((c >> 8) as u8);
        body.push((c & 0xFF) as u8);

        let frames = unwrap_notify(&body);
        assert!(
            frames.iter().any(|f| f == &vec![0xAA, 0x02, 0x64, 0x00, 0x64, 0x01]),
            "expected AA0264006401, got {:02X?}",
            frames
        );
    }
}
