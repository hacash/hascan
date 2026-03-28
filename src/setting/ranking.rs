use protocol::state::CoreStateRead;



/**
*
*/
pub fn update_ranking(setting: &mut ScanSettings, adrs: &AddressCache, state: &CoreStateRead) -> Rerr {

    for (addr, _) in adrs {
        let adr = Address::from_readable(addr)?;
        let Some(bls) = state.balance(&adr) else {
            continue
        };
        let zhu = bls.hacash.to_zhu_u128().unwrap_or(0) as u64;
        let sat = bls.satoshi.uint() as u64;
        let dia = bls.diamond.uint() as u64;
        // update
        update_one_rank(&mut setting.rank_zhu, &adr, zhu);
        update_one_rank(&mut setting.rank_sat, &adr, sat);
        update_one_rank(&mut setting.rank_dia, &adr, dia);
    }

    // truncate to 200
    let maxl: usize = 200;
    macro_rules! truncate {
        ($p: ident) => { {
            let mut zl = setting.$p.as_list().len();
            if zl > maxl {
                zl = maxl;
            }
            setting.$p.lists.truncate(zl);
            setting.$p.count = Uint1::from(zl as u8);
        } }
    }
    truncate!(rank_zhu);
    truncate!(rank_sat);
    truncate!(rank_dia);


    Ok(())

}


fn update_one_rank(rklist: &mut BalanceRankingList, adr: &Address, namt: u64) {
    let rklist = &mut rklist.lists;
    let nbls = Balance{addr: adr.clone(), amount: Uint8::from(namt)};
    // delete old
    rklist.retain(|x|x.addr!=*adr);
    if namt == 0 {
        return // do nother
    }
    // insert
    if rklist.len() == 0 {
        rklist.push(nbls);
        return
    }
    let mut istid = 0;
    let mut k = rklist.len() as i64 - 1;
    while k >= 0 {
        let i = k as usize;
        if namt <= *rklist[i].amount {
            istid = i + 1;
            break
        }
        k -= 1;
    }
    // print!("update_one_rank {} {} {} {}, ", adr.readable(), rklist.len(), istid, namt);
    if istid == rklist.len() {
        rklist.push(nbls);
    }else{
        rklist.insert(istid, nbls);
    }

    // println!("after len = {}", rklist.len());
    // ok
    
}
