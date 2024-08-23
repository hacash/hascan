

fn do_scan(scaner: &BlkScaner, setting: &mut ScanSettings, dbconn: &mut Connection, 
    adrary: &mut AddressCache,
    block: &dyn BlockRead, csto: CoreStoreDisk, csta: CoreStateDisk, msto: MintStoreDisk, msta: MintStateDisk,
) -> RetErr {

    // note
    let blk_info = hacash::protocol::block::create_recent_block_info(block);
    let hei = blk_info.height.uint();
    if hei % 1000 == 0 {
        println!("Scan block height {} finish.", hei);
    }
    //
    // let miner_id = 

    







    Ok(())
}
