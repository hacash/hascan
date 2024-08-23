

#[derive(Clone, Default)]
pub struct BlkScrConfig {

}


impl BlkScrConfig {

    pub fn new(ini: &IniObj) -> Ret<BlkScrConfig> {
        let cnf = BlkScrConfig {

        };

        Ok(cnf)
    }

}