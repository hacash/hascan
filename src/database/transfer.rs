
/**
* 
*/
pub fn record_coin_transfer(dbtx: &mut DBTransaction, adrs: &mut AddressCache, 
    trs: &dyn TransactionRead, setting: &mut ScanSettings, height: u64, blkts: u64,
    // diamovedate: &mut HashMap<DiamondName, u64>,
) -> DBResult<()> {

    let maddr = trs.address().unwrap();
    let aptrs = trs.addrlist();
    let (main_aid, main_acc) = record_addr_as_mut(dbtx, adrs, setting, &maddr, blkts)?;
    main_acc.used_fee += trs.fee().to_mei_unsafe();
    let actions = trs.actions();  
    for act in actions {
        record_one_action(dbtx, adrs, aptrs, act.as_ref(), setting, &maddr, 
            main_aid, height, blkts)?;
    }
    Ok(())
}


fn record_one_action(dbtx: &mut DBTransaction, adrs: &mut AddressCache, aptrs: &AddrOrList,
    act: &dyn Action, setting: &mut ScanSettings, maddr: &Address, main_aid: u64, 
    height: u64, blkts: u64,
    // diamovedate: &mut HashMap<DiamondName, u64>,
) -> DBResult<()> {

    const sqlirt: &str = "INSERT INTO coin_transfer 
        (height,from_aid,to_aid,coin_type,coin_amt) VALUES 
        (?1, ?2, ?3, ?4, ?5)";

    const sqlopt: &str = "INSERT INTO defi_operate 
        (height,kind,aid1,aid2,tarid,data) VALUES 
        (?1, ?2, ?3, ?4, ?5, ?6)";


    // target addr
    let kid = act.kind();

    /******** Hacash ********/

    if kid == HacToTransfer::kid() {

        let action = HacToTransfer::must(&act.serialize());
        let mut zhu = action.amt.to_zhu_unsafe();
        if zhu > 100_0000_00000000u64 as f64 {
            return Ok(()) // ingore super big amt, bugs
        }
        let zhu = zhu as u64;
        if zhu < 10000 {
            return Ok(()) // ingore < 1w zhu amt
        }
        let to_addr = action.to.real(aptrs).unwrap();
        let to_aid = record_addr_id(dbtx, adrs, setting, &to_addr, blkts)?;
        let mut stmt = dbtx.prepare_cached(sqlirt)?;
        stmt.insert((height, main_aid, to_aid, COINTY_ZHU, zhu))?;
        let active = record_current_active(setting, height);
        active.trszhu += 1;
        active.mvzhu += zhu;

    } else if kid == HacFromTransfer::kid() {

        let action = HacFromTransfer::must(&act.serialize());
        let zhu = action.amt.to_zhu_unsafe() as u64;
        if zhu < 10000 {
            return Ok(()) // ingore < 1w zhu amt
        }
        let from_addr = action.from.real(aptrs).unwrap();
        let from_aid = record_addr_id(dbtx, adrs, setting, &from_addr, blkts)?;
        let mut stmt = dbtx.prepare_cached(sqlirt)?;
        stmt.insert((height, from_aid, main_aid, COINTY_ZHU, zhu))?;
        let active = record_current_active(setting, height);
        active.trszhu += 1;
        active.mvzhu += zhu;

    } else if kid == HacFromToTransfer::kid() {

        let action = HacFromToTransfer::must(&act.serialize());
        let zhu = action.amt.to_zhu_unsafe() as u64;
        if zhu < 10000 {
            return Ok(()) // ingore < 1w zhu amt
        }
        let from_addr = action.from.real(aptrs).unwrap();
        let to_addr = action.to.real(aptrs).unwrap();
        let from_aid = record_addr_id(dbtx, adrs, setting, &from_addr, blkts)?;
        let to_aid = record_addr_id(dbtx, adrs, setting, &to_addr, blkts)?;
        let mut stmt = dbtx.prepare_cached(sqlirt)?;
        stmt.insert((height, from_aid, to_aid, COINTY_ZHU, zhu))?;
        let active = record_current_active(setting, height);
        active.trszhu += 1;
        active.mvzhu += zhu;

    /******** Satoshi ********/

    } else if kid == SatoshiToTransfer::kid() {

        let action = SatoshiToTransfer::must(&act.serialize());
        let to_addr = action.to.real(aptrs).unwrap();
        let to_aid = record_addr_id(dbtx, adrs, setting, &to_addr, blkts)?;
        let sat = action.satoshi.uint() as u64;
        let mut stmt = dbtx.prepare_cached(sqlirt)?;
        stmt.insert((height, main_aid, to_aid, COINTY_SAT, sat))?;
        let active = record_current_active(setting, height);
        active.trssat += 1;
        active.mvsat += sat;

    } else if kid == SatoshiFromTransfer::kid() {

        let action = SatoshiFromTransfer::must(&act.serialize());
        let from_addr = action.from.real(aptrs).unwrap();
        let from_aid = record_addr_id(dbtx, adrs, setting, &from_addr, blkts)?;
        let sat = action.satoshi.uint() as u64;
        let mut stmt = dbtx.prepare_cached(sqlirt)?;
        stmt.insert((height, from_aid, main_aid, COINTY_SAT, sat))?;
        let active = record_current_active(setting, height);
        active.trssat += 1;
        active.mvsat += sat;

    } else if kid == SatoshiFromToTransfer::kid() {

        let action = SatoshiFromToTransfer::must(&act.serialize());
        let from_addr = action.from.real(aptrs).unwrap();
        let to_addr = action.to.real(aptrs).unwrap();
        let from_aid = record_addr_id(dbtx, adrs, setting, &from_addr, blkts)?;
        let to_aid = record_addr_id(dbtx, adrs, setting, &to_addr, blkts)?;
        let sat = action.satoshi.uint() as u64;
        let mut stmt = dbtx.prepare_cached(sqlirt)?;
        stmt.insert((height, from_aid, to_aid, COINTY_SAT, sat))?;
        let active = record_current_active(setting, height);
        active.trssat += 1;
        active.mvsat += sat;
    
    /******** Diamond ********/

    } else if kid == DiamondSingleTransfer::kid() {

        let action = DiamondSingleTransfer::must(&act.serialize());
        let to_addr = action.to.real(aptrs).unwrap();
        let to_aid = record_addr_id(dbtx, adrs, setting, &to_addr, blkts)?;
        let dia = 1 as u64; // only one
        let mut stmt = dbtx.prepare_cached(sqlirt)?;
        stmt.insert((height, main_aid, to_aid, COINTY_DIA, dia))?;
        let active = record_current_active(setting, height);
        active.trsdia += 1;
        active.mvdia += dia;

        // diamovedate.insert(action.diamond, blkts);


    } else if kid == DiamondFromTransfer::kid() {

        let action = DiamondFromTransfer::must(&act.serialize());
        let from_addr = action.from.real(aptrs).unwrap();
        let from_aid = record_addr_id(dbtx, adrs, setting, &from_addr, blkts)?;
        let dia = action.diamonds.count().uint() as u64;
        let mut stmt = dbtx.prepare_cached(sqlirt)?;
        stmt.insert((height, from_aid, main_aid, COINTY_DIA, dia))?;
        let active = record_current_active(setting, height);
        active.trsdia += 1;
        active.mvdia += dia;

        // for dia in action.diamonds.list() {
        //     diamovedate.insert(*dia, blkts);
        // }

    } else if kid == DiamondToTransfer::kid() {

        let action = DiamondToTransfer::must(&act.serialize());
        let to_addr = action.to.real(aptrs).unwrap();
        let to_aid = record_addr_id(dbtx, adrs, setting, &to_addr, blkts)?;
        let dia = action.diamonds.count().uint() as u64;
        let mut stmt = dbtx.prepare_cached(sqlirt)?;
        stmt.insert((height, main_aid, to_aid, COINTY_DIA, dia))?;
        let active = record_current_active(setting, height);
        active.trsdia += 1;
        active.mvdia += dia;

        // for dia in action.diamonds.list() {
        //     diamovedate.insert(*dia, blkts);
        // }


    } else if kid == DiamondFromToTransfer::kid() {

        let action = DiamondFromToTransfer::must(&act.serialize());
        let from_addr = action.from.real(aptrs).unwrap();
        let from_aid = record_addr_id(dbtx, adrs, setting, &from_addr, blkts)?;
        let to_addr = action.to.real(aptrs).unwrap();
        let to_aid = record_addr_id(dbtx, adrs, setting, &to_addr, blkts)?;
        let dia = action.diamonds.count().uint() as u64;
        let mut stmt = dbtx.prepare_cached(sqlirt)?;
        stmt.insert((height, from_aid, to_aid, COINTY_DIA, dia))?;
        let active = record_current_active(setting, height);
        active.trsdia += 1;
        active.mvdia += dia;

        // for dia in action.diamonds.list() {
        //     diamovedate.insert(*dia, blkts);
        // }


    /******** Diamond mint ********/

    } else if kid == DiamondMint::kid() {

        let action = DiamondMint::must(&act.serialize());
        let miner_addr = &action.head.address;
        let (_, accobj) = record_addr_as_mut(dbtx, adrs, setting, miner_addr, blkts)?;
        accobj.minted_diamond += 1;


    /******** Channel Operate ********/


    } else if kid == ChannelOpen::kid() {

        let action = ChannelOpen::must(&act.serialize());
        let left_addr = action.left_bill.address;
        let left_aid = record_addr_id(dbtx, adrs, setting, &left_addr, blkts)?;
        let right_addr = action.right_bill.address;
        let right_aid = record_addr_id(dbtx, adrs, setting, &right_addr, blkts)?;
        let tar_id = action.channel_id.to_vec();
        let notes = format!("{},{}", 
            action.left_bill.amount.to_fin_string(),
            action.right_bill.amount.to_fin_string(),
        );
        let mut stmt = dbtx.prepare_cached(sqlopt)?;
        stmt.insert((height, OPTY_CH_OPEN, left_aid, right_aid, tar_id, notes))?;

    } else if kid == ChannelClose::kid() {

        let action = ChannelClose::must(&act.serialize());
        let tar_id = action.channel_id.to_vec();
        let mut stmt = dbtx.prepare_cached(sqlopt)?;
        stmt.insert((height, OPTY_CH_CLOSE, main_aid, 0, tar_id, ""))?;

    }
    
    


    Ok(())

}