//! Pure parsing of BLE advertisement data used by the Baseus discovery flow.

pub const BASEUS_SERVICE_UUID: &str = "53527AA4-29F7-AE11-4E74-997334782568";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Advertisement {
    pub service_uuids: Vec<String>,
    pub manufacturer_data: Vec<u8>,
    pub local_name: Option<String>,
}

pub fn parse_advertisement(data: &[u8]) -> Advertisement {
    let mut result = Advertisement {
        service_uuids: Vec::new(),
        manufacturer_data: Vec::new(),
        local_name: None,
    };
    let mut offset = 0;
    while offset < data.len() {
        let length = data[offset] as usize;
        offset += 1;
        if length == 0 {
            continue;
        }
        let end = match offset.checked_add(length) {
            Some(end) if end <= data.len() => end,
            _ => break,
        };
        let ad_type = data[offset];
        let payload = &data[offset + 1..end];
        match ad_type {
            0x02 | 0x03 => {
                for chunk in payload.chunks_exact(2) {
                    result
                        .service_uuids
                        .push(format!("{:02X}{:02X}", chunk[1], chunk[0]));
                }
            }
            0x06 | 0x07 if payload.len() == 16 => {
                result.service_uuids.push(format_uuid128(payload));
            }
            0x08 | 0x09 => {
                result.local_name = String::from_utf8(payload.to_vec()).ok();
            }
            0xFF => {
                result.manufacturer_data = payload.to_vec();
            }
            _ => {}
        }
        offset = end;
    }
    result
}

fn format_uuid128(bytes: &[u8]) -> String {
    let mut b = [0u8; 16];
    b.copy_from_slice(bytes);
    b[..4].reverse();
    b[4..6].reverse();
    b[6..8].reverse();
    format!(
        "{:02X}{:02X}{:02X}{:02X}-{:02X}{:02X}-{:02X}{:02X}-{:02X}{:02X}-{:02X}{:02X}{:02X}{:02X}{:02X}{:02X}",
        b[0], b[1], b[2], b[3], b[4], b[5], b[6], b[7], b[8], b[9], b[10], b[11], b[12], b[13], b[14], b[15]
    )
}

pub fn canonical_serial(manufacturer: &[u8], reverse_bytes: bool) -> Option<String> {
    if manufacturer.len() < 12 {
        return None;
    }
    let format_part = |part: &[u8]| {
        if reverse_bytes {
            part.iter()
                .rev()
                .map(|b| format!("{b:02X}"))
                .collect::<String>()
        } else {
            part.iter().map(|b| format!("{b:02X}")).collect::<String>()
        }
    };
    Some(format!(
        "{}_{}",
        format_part(&manufacturer[..6]),
        format_part(&manufacturer[6..12])
    ))
}

pub fn has_required_service(service_uuids: &[&str], required: bool) -> bool {
    !required
        || service_uuids
            .iter()
            .any(|uuid| uuid.eq_ignore_ascii_case(BASEUS_SERVICE_UUID))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::protocol::{identify_model, ModelInfo};

    #[test]
    fn parses_ad_structures_and_service_uuids() {
        let data = [
            3, 0x03, 0x68, 0x56, 3, 0x03, 0xAA, 0xBB, 4, 0xFF, 0x01, 0x02, 0x03,
        ];
        let parsed = parse_advertisement(&data);
        assert_eq!(
            parsed.service_uuids,
            vec!["5668", "BBAA"],
            "UUIDs are normalized to advertised bytes"
        );
        assert_eq!(parsed.manufacturer_data, vec![0x01, 0x02, 0x03]);
    }

    #[test]
    fn derives_canonical_serial_from_two_six_byte_values() {
        let manufacturer = [
            0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x10, 0x11, 0x12, 0x13, 0x14, 0x15,
        ];
        let serial = canonical_serial(&manufacturer, false).unwrap();
        assert_eq!(serial, "010203040506_101112131415");
    }

    #[test]
    fn reverses_model_specific_serial_byte_order() {
        let manufacturer = [
            0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x10, 0x11, 0x12, 0x13, 0x14, 0x15,
        ];
        let serial = canonical_serial(&manufacturer, true).unwrap();
        assert_eq!(serial, "060504030201_151413121110");
    }

    #[test]
    fn rejects_required_service_uuid_when_missing() {
        assert!(!has_required_service(
            &["0000180F-0000-1000-8000-00805F9B34FB"],
            true
        ));
        assert!(has_required_service(&[BASEUS_SERVICE_UUID], true));
        assert!(has_required_service(&[], false));
    }

    #[test]
    fn keeps_model_catalog_fields_for_product_presentation() {
        let model: ModelInfo = identify_model("Baseus Bass BP1 Pro").unwrap();
        assert_eq!(model.image_provenance, "fallback");
        assert!(model.capabilities.bass_boost);
        assert_eq!(
            model.transport.service_uuid.as_deref(),
            Some(BASEUS_SERVICE_UUID)
        );
    }
}
