


#[derive(Clone, Default)]
pub struct AddressSto {
    pub id: i64,
    // address: String,
    pub minted_diamond: u32,
    pub block_reward: u64,
    pub used_fee: f64,
    // timestamp > 0 mean is insert to database
    pub timestamp: u64,
}

impl AddressSto {
    pub fn new(aid: i64) -> AddressSto {
        AddressSto {
            id: aid,
            ..Default::default()
        }
    }
}



pub type AddressCache = HashMap<String, AddressSto>; // address => sto



