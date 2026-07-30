pub fn update_chain_active(setting: &mut ScanSettings, newadr: u32, height: u64) -> Rerr {
    let secc = record_current_active(setting, height);
    secc.newadr = Uint4::from(
        secc.newadr
            .uint()
            .checked_add(newadr)
            .ok_or_else(|| sys::Error::fault("hascan new address count overflow"))?,
    );
    Ok(())
}

pub fn record_current_active(setting: &mut ScanSettings, height: u64) -> &mut ActiveItem {
    // defs
    let sechei: usize = 2000; // one week
    let maxsec: usize = 25; // half year
    let cursec = height.saturating_sub(1) / sechei as u64 + 1;

    let (count, actives) = {
        let chain_active = &mut setting.chain_active;
        (&mut chain_active.count, &mut chain_active.lists)
    };

    let same_cursec = actives
        .first()
        .map(|item| item.secnum.uint() as u64 == cursec)
        .unwrap_or(false);

    if !same_cursec {
        // new
        let acone = ActiveItem {
            secnum: Uint4::from(cursec as u32),
            ..Default::default()
        };
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
