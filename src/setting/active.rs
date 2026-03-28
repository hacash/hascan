
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
    let cursec = height.saturating_sub(1) / sechei as u64 + 1;

    let (count, actives) = {
        let chain_active = &mut setting.chain_active;
        (&mut chain_active.count, &mut chain_active.lists)
    };

    let same_cursec = actives.first()
        .map(|item| item.secnum.uint() as u64 == cursec)
        .unwrap_or(false);

    if !same_cursec {
        // new
        let mut acone = ActiveItem::default();
        acone.secnum = Uint4::from(cursec as u32);
        // create
        if actives.is_empty() {
            actives.push(acone.clone());
        }
        if actives[0].secnum.uint() as u64 != cursec {
            actives.insert(0, acone);
        }
        // max truncate
        if actives.len() > maxsec {
            actives.truncate(maxsec);
        }
    }

    *count = Uint1::from(actives.len() as u8);
    actives.get_mut(0).unwrap()
}
