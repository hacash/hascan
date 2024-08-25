

#[derive(Clone, Default)]
pub struct BlkScrConfig {
    pub synchronous: String, // NORMAL, FULL, OFF
    pub delaysavesetting: u64,
    pub listen: u16,
}


impl BlkScrConfig {

    pub fn new(ini: &IniObj) -> Ret<BlkScrConfig> {

        let sec = &ini_section(ini, "hascan"); // default = root
        let synchronous = ini_must(sec, "synchronous", "NORMAL");
        let delaysavesetting = ini_must_u64(sec, "delaysavesetting", 0);
        let listen = ini_must_u64(sec, "listen", 8087) as u16;


        let cnf = BlkScrConfig {
            synchronous,
            delaysavesetting,
            listen,
        };
        Ok(cnf)
    }

}