
/**
*
*/
pub fn update_chain_active(setting: &mut ScanSettings, adrs: &AddressCache, height: u64) -> Rerr {
    let secc = record_current_active(setting, height);
    // update
    // new addr
    let mut newadr = 0u32;
    for (_, sto) in adrs {
        if sto.timestamp > 0 {
            newadr += 1; // addr is new
        }
    }
    secc.newadr += newadr;
    Ok(())
}


pub fn record_current_active<'a>(setting: &'a mut ScanSettings, height: u64) -> &'a mut ActiveItem {
    // defs
    let sechei: usize = 2000; // one week
    let maxsec: usize = 25; // half year
    let cursec = (height-1) / sechei as u64 + 1;
    let actives = setting.chain_active.as_mut();
    if actives.len() > 0 &&  actives[0].secnum.uint() as u64 == cursec {
        return setting.chain_active.as_mut().get_mut(0).unwrap()
    }else{
        // new
        let mut acone = ActiveItem::default();
        acone.secnum = Uint4::from(cursec as u32);
        // create
        if actives.len() == 0 {
            actives.push(acone.clone());
        }
        if actives[0].secnum.uint() as u64 != cursec {
            actives.insert(0, acone);
        }
        // max truncate
        let mut rsl = actives.len();
        if rsl > maxsec {
            rsl = maxsec;
        }
        actives.truncate(rsl);
        setting.chain_active.count = Uint1::from(rsl as u8);
        // ok
        setting.chain_active.as_mut().get_mut(0).unwrap()
    }
}