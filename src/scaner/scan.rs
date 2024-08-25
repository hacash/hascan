

fn do_scan(scaner: &BlkScaner, setting: &mut ScanSettings, dbconn: &mut Connection, 
    adrary: &mut AddressCache,
    block: &dyn BlockRead, csto: CoreStoreDisk, csta: CoreStateDisk, msto: MintStoreDisk, msta: MintStateDisk,
    // diamovedate: &mut HashMap<DiamondName, u64>,
) -> RetErr {
    macro_rules! err {
        ($v: expr) => {
            $v.map_err(|e|e.to_string())?
        }
    }
    // db tx
    let mut dbtx = err!(dbconn.transaction());
    // note
    let blk_info = hacash::protocol::block::create_recent_block_info(block);
    let hei = blk_info.height.uint() as u64;
    let blkts = blk_info.time.uint() as u64;
    if hei % 1000 == 0 {
        println!("Scan block height {} finish.", hei);
    }
    // add erward to miner
    let miner_addr = blk_info.miner.readable();
    let miner_id =  err!(record_addr_id(&mut dbtx, adrary, setting, &blk_info.miner, hei));
    adrary.get_mut(&miner_addr).unwrap().block_reward += blk_info.reward.to_mei_unsafe() as u64;
    // chain active
    let active = record_current_active(setting, hei);
    // record coin transfer
    let trslist = block.transactions();
    let txs = trslist.len();
    active.txs += (txs - 1) as u32; // stats txs
    drop(active);
    for i in 1..txs { // ingore coinbase
        err!(record_coin_transfer(&mut dbtx, adrary, trslist[i].as_read(), setting, hei, blkts));
    }
    // insert address to database
    err!(insert_update_addr(&mut dbtx, adrary));
    //dbtx
    err!(dbtx.commit());
    // ranking
    update_ranking(setting, adrary, &csta)?;
    update_chain_active(setting, adrary, hei)?;
    // save settings
    let stsvt = scaner.cnf.delaysavesetting as u64;
    if stsvt == 0 {
        super::save_setting(setting); // save it now
    }else{
        let nowt = hacash::sys::curtimes();
        let mut prvt = scaner.prevsavetime.lock().unwrap();
        if nowt - *prvt > stsvt {
            super::save_setting(setting); // save it now
            *prvt = nowt;
        }
    }
    Ok(())
}
