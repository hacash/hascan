
/**
* 
*/
pub fn record_coin_transfer(dbtx: &mut DBTransaction, adrs: &mut AddressCache, 
    trs: &dyn TransactionRead, setting: &mut ScanSettings, height: u64, blkts: u64,
    // diamovedate: &mut HashMap<DiamondName, u64>,
) -> DBResult<()> {

    let maddr = trs.main();
    let aptrs = trs.addrs();
    let (main_aid, main_acc) = record_addr_as_mut(dbtx, adrs, setting, &maddr, blkts)?;
    main_acc.used_fee += unsafe { trs.fee().to_unit_float(UNIT_MEI) };
    let actions = trs.actions();  
    for act in actions {
        record_one_action(dbtx, adrs, &aptrs, act.as_ref(), setting, &maddr, 
            main_aid, height, blkts)?;
    }
    Ok(())
}


fn record_one_action(dbtx: &mut DBTransaction, adrs: &mut AddressCache, aptrs: &Vec<Address>,
    act: &dyn Action, setting: &mut ScanSettings, _maddr: &Address, main_aid: u64, 
    height: u64, blkts: u64,
    // diamovedate: &mut HashMap<DiamondName, u64>,
) -> DBResult<()> {

    let sqlirt: &str = "INSERT INTO coin_transfer 
        (height,from_aid,to_aid,coin_type,coin_amt) VALUES 
        (?1, ?2, ?3, ?4, ?5)";

    let sqlopt: &str = "INSERT INTO defi_operate 
        (height,kind,aid1,aid2,tarid,data) VALUES 
        (?1, ?2, ?3, ?4, ?5, ?6)";


    // target addr
    let kid = act.kind();

    /******** Hacash ********/

    if kid == HacToTrs::KIND {

        let action = HacToTrs::must(&act.serialize());
        let zhu = action.hacash.to_zhu_u128().unwrap_or(0);
        if zhu > 100_0000_00000000u128 {
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

    } else if kid == HacFromTrs::KIND {

        let action = HacFromTrs::must(&act.serialize());
        let zhu = action.hacash.to_zhu_u128().unwrap_or(0) as u64;
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

    } else if kid == HacFromToTrs::KIND {

        let action = HacFromToTrs::must(&act.serialize());
        let zhu = action.hacash.to_zhu_u128().unwrap_or(0) as u64;
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

    } else if kid == SatToTrs::KIND {

        let action = SatToTrs::must(&act.serialize());
        let to_addr = action.to.real(aptrs).unwrap();
        let to_aid = record_addr_id(dbtx, adrs, setting, &to_addr, blkts)?;
        let sat = action.satoshi.uint() as u64;
        let mut stmt = dbtx.prepare_cached(sqlirt)?;
        stmt.insert((height, main_aid, to_aid, COINTY_SAT, sat))?;
        let active = record_current_active(setting, height);
        active.trssat += 1;
        active.mvsat += sat;

    } else if kid == SatFromTrs::KIND {

        let action = SatFromTrs::must(&act.serialize());
        let from_addr = action.from.real(aptrs).unwrap();
        let from_aid = record_addr_id(dbtx, adrs, setting, &from_addr, blkts)?;
        let sat = action.satoshi.uint() as u64;
        let mut stmt = dbtx.prepare_cached(sqlirt)?;
        stmt.insert((height, from_aid, main_aid, COINTY_SAT, sat))?;
        let active = record_current_active(setting, height);
        active.trssat += 1;
        active.mvsat += sat;

    } else if kid == SatFromToTrs::KIND {

        let action = SatFromToTrs::must(&act.serialize());
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

    } else if kid == DiaSingleTrs::KIND {

        let action = DiaSingleTrs::must(&act.serialize());
        let to_addr = action.to.real(aptrs).unwrap();
        let to_aid = record_addr_id(dbtx, adrs, setting, &to_addr, blkts)?;
        let dia = 1 as u64; // only one
        let mut stmt = dbtx.prepare_cached(sqlirt)?;
        stmt.insert((height, main_aid, to_aid, COINTY_DIA, dia))?;
        let active = record_current_active(setting, height);
        active.trsdia += 1;
        active.mvdia += dia;

        // diamovedate.insert(action.diamond, blkts);


    } else if kid == DiaFromTrs::KIND {

        let action = DiaFromTrs::must(&act.serialize());
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

    } else if kid == DiaToTrs::KIND {

        let action = DiaToTrs::must(&act.serialize());
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


    } else if kid == DiaFromToTrs::KIND {

        let action = DiaFromToTrs::must(&act.serialize());
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

    } else if kid == DiamondMint::KIND {

        let action = DiamondMint::must(&act.serialize());
        let miner_addr = &action.d.address;
        let (_, accobj) = record_addr_as_mut(dbtx, adrs, setting, miner_addr, blkts)?;
        accobj.minted_diamond += 1;


    /******** Channel Operate ********/


    } else if kid == ChannelOpen::KIND {

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

    } else if kid == ChannelClose::KIND {

        let action = ChannelClose::must(&act.serialize());
        let tar_id = action.channel_id.to_vec();
        let mut stmt = dbtx.prepare_cached(sqlopt)?;
        stmt.insert((height, OPTY_CH_CLOSE, main_aid, 0, tar_id, ""))?;

    }
    
    


    Ok(())

}