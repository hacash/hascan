use std::collections::HashMap;

use field::*;
use sys::*;

include!("address.rs");
include!("setting.rs");
include!("ranking.rs");
include!("active.rs");

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn settings_keep_the_legacy_field_order() {
        let settings = ScanSettings {
            height: Uint5::from(0x0102030405),
            auto_inc_address_id: Uint5::from(0x060708090a),
            chain_active: ChainActiveList {
                count: Uint1::from(1),
                lists: vec![ActiveItem {
                    secnum: Uint4::from(1),
                    newadr: Uint4::from(2),
                    txs: Uint4::from(3),
                    trszhu: Uint4::from(4),
                    trssat: Uint4::from(5),
                    trsdia: Uint4::from(6),
                    mvzhu: Uint16::from(7),
                    mvsat: Uint12::from(8),
                    mvdia: Uint8::from(9),
                }],
            },
            ..Default::default()
        };
        let encoded = settings.encode();
        assert_eq!(&encoded[..10], &[1, 2, 3, 4, 5, 6, 7, 8, 9, 10]);
        let (decoded, used) = ScanSettings::decode(&encoded).unwrap();
        assert_eq!(used, encoded.len());
        assert_eq!(decoded, settings);
        assert_eq!(decoded.encode(), encoded);
    }
}
