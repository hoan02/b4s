//! Bounded, session-scoped diagnostic details for the active BLE link.

const MAX_PREVIEW_BYTES: usize = 64;

#[derive(Default)]
pub(super) struct LinkDiagnostics {
    pub(super) notify_count: u64,
    pub(super) tx_count: u64,
    pub(super) last_notify_ms: Option<u64>,
    pub(super) last_tx_ms: Option<u64>,
    pub(super) last_rx_hex: Option<String>,
    pub(super) last_tx_hex: Option<String>,
}

impl LinkDiagnostics {
    pub(super) fn record_rx(&mut self, data: &[u8], timestamp_ms: u64) {
        self.notify_count = self.notify_count.saturating_add(1);
        self.last_notify_ms = Some(timestamp_ms);
        self.last_rx_hex = Some(hex_preview(data));
    }

    pub(super) fn record_tx(&mut self, data: &[u8], timestamp_ms: u64) {
        self.tx_count = self.tx_count.saturating_add(1);
        self.last_tx_ms = Some(timestamp_ms);
        self.last_tx_hex = Some(hex_preview(data));
    }
}

fn hex_preview(data: &[u8]) -> String {
    let shown = &data[..data.len().min(MAX_PREVIEW_BYTES)];
    let mut preview = shown
        .iter()
        .map(|byte| format!("{byte:02X}"))
        .collect::<Vec<_>>()
        .join(" ");

    let omitted = data.len().saturating_sub(shown.len());
    if omitted > 0 {
        preview.push_str(&format!(" … (+{omitted} bytes)"));
    }

    preview
}

#[cfg(test)]
mod tests {
    use super::{hex_preview, LinkDiagnostics, MAX_PREVIEW_BYTES};

    #[test]
    fn diagnostic_preview_keeps_short_payloads_intact() {
        assert_eq!(hex_preview(&[0x00, 0xA5, 0xFF]), "00 A5 FF");
    }

    #[test]
    fn diagnostic_preview_caps_large_payloads_and_reports_omitted_bytes() {
        let payload = vec![0xAB; MAX_PREVIEW_BYTES + 3];
        let preview = hex_preview(&payload);

        assert!(preview.starts_with(&"AB ".repeat(MAX_PREVIEW_BYTES - 1)));
        assert!(preview.ends_with(" … (+3 bytes)"));
        assert!(preview.len() < MAX_PREVIEW_BYTES * 3 + 32);
    }

    #[test]
    fn diagnostic_cache_records_counts_and_replaces_last_preview() {
        let mut diagnostics = LinkDiagnostics::default();
        diagnostics.record_rx(&[0x01], 10);
        diagnostics.record_rx(&[0x02, 0x03], 20);
        diagnostics.record_tx(&[0xBA], 30);

        assert_eq!(diagnostics.notify_count, 2);
        assert_eq!(diagnostics.tx_count, 1);
        assert_eq!(diagnostics.last_notify_ms, Some(20));
        assert_eq!(diagnostics.last_tx_ms, Some(30));
        assert_eq!(diagnostics.last_rx_hex.as_deref(), Some("02 03"));
        assert_eq!(diagnostics.last_tx_hex.as_deref(), Some("BA"));
    }
}
