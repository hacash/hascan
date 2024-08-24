

fn do_scan(scaner: &BlkScaner, setting: &mut ScanSettings, dbconn: &mut Connection, 
    adrary: &mut AddressCache,
    block: &dyn BlockRead, csto: CoreStoreDisk, csta: CoreStateDisk, msto: MintStoreDisk, msta: MintStateDisk,
) -> RetErr {
    macro_rules! maperr {
        ($v: expr) => {
            $v.map_err(|e|e.to_string())?
        }
    }
    // db tx
    let mut dbtx = maperr!(dbconn.transaction());
    // note
    let blk_info = hacash::protocol::block::create_recent_block_info(block);
    let hei = blk_info.height.uint() as u64;
    if hei % 1000 == 0 {
        println!("Scan block height {} finish.", hei);
    }
    let (_, miner_id) =  maperr!(record_addr_id(&mut dbtx, adrary, setting, blk_info.miner.readable(), hei));
    // chain active
    let active = record_current_active(setting, hei);
    // record coin transfer
    let trslist = block.transactions();
    let txs = trslist.len();
    active.txs += (txs - 1) as u32; // stats txs
    for i in 1..txs { // ingore coinbase
        maperr!(record_coin_transfer(&mut dbtx, trslist[i].as_read(), active));
    }
    // insert address to database
    maperr!(insert_update_addr(&mut dbtx, adrary));
    //dbtx
    maperr!(dbtx.commit());
    // ranking
    update_ranking(setting, adrary, &csta)?;
    // save settings
    super::save_setting(setting);
    Ok(())
}
