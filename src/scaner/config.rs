#[derive(Clone, Default)]
pub struct BlkScrConfig {
    pub datadir: String,
    pub synchronous: String, // NORMAL, FULL, OFF
    pub scan_batch_blocks: u64,
    pub stop_at_height: u64,
}

impl BlkScrConfig {
    pub fn new(ini: &IniObj) -> Ret<BlkScrConfig> {
        let sec = &ini_section(ini, "hascan"); // default = root
        let datadir = ini_must(sec, "datadir", "hacash_scan_data");
        let synchronous = ini_must(sec, "synchronous", "NORMAL").to_ascii_uppercase();
        let scan_batch_blocks = ini_must_u64(sec, "scan_batch_blocks", 200).clamp(1, 1_000);
        let stop_at_height = ini_must_u64(sec, "stop_at_height", 0);
        if !matches!(synchronous.as_str(), "OFF" | "NORMAL" | "FULL" | "EXTRA") {
            return sys::errf!("config [hascan].synchronous must be OFF, NORMAL, FULL, or EXTRA");
        }

        Ok(BlkScrConfig {
            datadir,
            synchronous,
            scan_batch_blocks,
            stop_at_height,
        })
    }
}
