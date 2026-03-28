
fn do_scan(scaner: &BlkScaner, setting: &mut ScanSettings, dbconn: &mut Connection, 
    adrary: &mut AddressCache,
    block: &dyn BlockRead, csta: CoreStateRead, _csto: BlockStore,
    // diamovedate: &mut HashMap<DiamondName, u64>,
) -> Rerr {
    macro_rules! err {
        ($v: expr) => {
            $v.map_err(|e|e.to_string())?
        }
    }
    // db tx
    let mut dbtx = err!(dbconn.transaction());
    // note
    let blk_info = create_recent_block_info(block);
    let hei = blk_info.height;
    let blkts = blk_info.time;
    let last_hei = setting.height.uint();
    if hei <= last_hei {
        return Ok(()) // dedup by scanned height
    }
    if hei % 1000 == 0 {
        println!("Scan block height {} finish.", hei);
    }
    // add erward to miner
    let (_miner_id, miner_acc) =  err!(record_addr_as_mut(&mut dbtx, adrary, setting, &blk_info.miner, blkts));
    miner_acc.block_reward += blk_info.reward.to_mei_u128().unwrap_or(0) as u64;
    // chain active
    let active = record_current_active(setting, hei);
    // record coin transfer
    let trslist = block.transactions();
    let txs = trslist.len();
    active.txs += txs.saturating_sub(1) as u32; // stats txs (ignore coinbase)
    let _ = active; // drop(active)
    for trs in trslist.iter().skip(1) { // ingore coinbase
        err!(record_coin_transfer(&mut dbtx, adrary, trs.as_read(), setting, hei, blkts));
    }
    // insert address to database
    err!(insert_update_addr(&mut dbtx, adrary));
    // persist scan dedup watermark in the same transaction
    err!(save_scan_height_tx(&mut dbtx, hei));
    //dbtx
    err!(dbtx.commit());
    // ranking
    update_ranking(setting, adrary, &csta)?;
    update_chain_active(setting, adrary, hei)?;
    setting.height = Uint5::from(hei);
    // save settings
    let stsvt = scaner.cnf.delaysavesetting;
    if stsvt == 0 {
        let _ = super::save_setting(&scaner.cnf.datadir, setting); // save it now
    }else{
        let nowt = sys::curtimes();
        let mut prvt = scaner.prevsavetime.lock().unwrap();
        if nowt - *prvt > stsvt {
            let _ = super::save_setting(&scaner.cnf.datadir, setting); // save it now
            *prvt = nowt;
        }
    }
    Ok(())
}
