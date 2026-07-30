
fn add_uint4(value: &mut Uint4, add: u32, name: &str) -> DBResult<()> {
    let next = value
        .uint()
        .checked_add(add)
        .and_then(Uint4::from_checked)
        .ok_or_else(|| db_overflow(name))?;
    *value = next;
    Ok(())
}

fn add_uint8(value: &mut Uint8, add: u64, name: &str) -> DBResult<()> {
    let next = value
        .uint()
        .checked_add(add)
        .and_then(Uint8::from_checked)
        .ok_or_else(|| db_overflow(name))?;
    *value = next;
    Ok(())
}

fn add_uint12(value: &mut Uint12, add: u64, name: &str) -> DBResult<()> {
    let next = value
        .uint()
        .checked_add(add as u128)
        .and_then(Uint12::from_checked)
        .ok_or_else(|| db_overflow(name))?;
    *value = next;
    Ok(())
}

fn add_uint16(value: &mut Uint16, add: u64, name: &str) -> DBResult<()> {
    let next = value
        .uint()
        .checked_add(add as u128)
        .and_then(Uint16::from_checked)
        .ok_or_else(|| db_overflow(name))?;
    *value = next;
    Ok(())
}

pub fn record_coin_transfer(dbtx: &mut DBTransaction, adrs: &mut AddressCache, 
    trs: &dyn Transaction, setting: &mut ScanSettings, height: u64, blkts: u64,
) -> DBResult<()> {

    let maddr = trs.main();
    let aptrs = trs.addrs();
    let (main_aid, main_acc) = record_addr_as_mut(dbtx, adrs, setting, &maddr, blkts)?;
    main_acc.used_fee += trs.fee().to_unit_float(UNIT_MEI);
    let actions = trs.actions();  
    for act in actions {
        record_one_action(
            dbtx,
            adrs,
            &aptrs,
            act.as_ref(),
            setting,
            &maddr,
            main_aid,
            height,
            blkts,
        )?;
    }
    Ok(())
}


#[allow(clippy::too_many_arguments)]
fn record_one_action(dbtx: &mut DBTransaction, adrs: &mut AddressCache, aptrs: &[Address],
    act: &dyn Action, setting: &mut ScanSettings, main_addr: &Address, main_aid: u64,
    height: u64, blkts: u64,
) -> DBResult<()> {
    macro_rules! action_ref {
        ($ty:ty) => {
            act.as_any()
                .downcast_ref::<$ty>()
                .ok_or_else(|| db_fault("hascan action kind/type mismatch"))?
        };
    }
    macro_rules! real_addr {
        ($value:expr) => {
            $value
                .real(aptrs)
                .map_err(|e| db_fault(format!("hascan action address failed: {e}")))?
        };
    }

    let sqlirt: &str = "INSERT INTO coin_transfer 
        (height,from_aid,to_aid,coin_type,coin_amt) VALUES 
        (?1, ?2, ?3, ?4, ?5)";

    let sqlopt: &str = "INSERT INTO defi_operate 
        (height,kind,aid1,aid2,tarid,data) VALUES 
        (?1, ?2, ?3, ?4, ?5, ?6)";


    // target addr
    let kid = act.kind();

    if record_ecosystem_action(dbtx, adrs, setting, act, main_addr, main_aid, height, blkts)? {
        return Ok(());
    }

    /******** Hacash ********/

    if kid == HacToTrs::KIND {

        let action = act
            .as_any()
            .downcast_ref::<HacToTrs>()
            .ok_or_else(|| db_fault("hascan action kind/type mismatch"))?;
        let zhu = action
            .hacash
            .to_zhu_u128()
            .map_err(|e| db_fault(format!("hascan HAC amount conversion failed: {e}")))?;
        if zhu > 100_000_000_000_000_u128 {
            return Ok(()) // ingore super big amt, bugs
        }
        let zhu = zhu as u64;
        if zhu < 10000 {
            return Ok(()) // ingore < 1w zhu amt
        }
        let to_addr = action
            .to
            .real(aptrs)
            .map_err(|e| db_fault(format!("hascan action address failed: {e}")))?;
        let to_aid = record_addr_id(dbtx, adrs, setting, &to_addr, blkts)?;
        let mut stmt = dbtx.prepare_cached(sqlirt)?;
        stmt.insert((height, main_aid, to_aid, COINTY_ZHU, zhu))?;
        let active = record_current_active(setting, height);
        add_uint4(&mut active.trszhu, 1, "HAC transfer count")?;
        add_uint16(&mut active.mvzhu, zhu, "HAC transfer amount")?;

    } else if kid == HacFromTrs::KIND {

        let action = act
            .as_any()
            .downcast_ref::<HacFromTrs>()
            .ok_or_else(|| db_fault("hascan action kind/type mismatch"))?;
        let zhu = action
            .hacash
            .to_zhu_u128()
            .map_err(|e| db_fault(format!("hascan HAC amount conversion failed: {e}")))?;
        if zhu > 100_000_000_000_000_u128 {
            return Ok(());
        }
        let zhu = zhu as u64;
        if zhu < 10000 {
            return Ok(()) // ingore < 1w zhu amt
        }
        let from_addr = action
            .from
            .real(aptrs)
            .map_err(|e| db_fault(format!("hascan action address failed: {e}")))?;
        let from_aid = record_addr_id(dbtx, adrs, setting, &from_addr, blkts)?;
        let mut stmt = dbtx.prepare_cached(sqlirt)?;
        stmt.insert((height, from_aid, main_aid, COINTY_ZHU, zhu))?;
        let active = record_current_active(setting, height);
        add_uint4(&mut active.trszhu, 1, "HAC transfer count")?;
        add_uint16(&mut active.mvzhu, zhu, "HAC transfer amount")?;

    } else if kid == HacFromToTrs::KIND {

        let action = act
            .as_any()
            .downcast_ref::<HacFromToTrs>()
            .ok_or_else(|| db_fault("hascan action kind/type mismatch"))?;
        let zhu = action
            .hacash
            .to_zhu_u128()
            .map_err(|e| db_fault(format!("hascan HAC amount conversion failed: {e}")))?;
        if zhu > 100_000_000_000_000_u128 {
            return Ok(());
        }
        let zhu = zhu as u64;
        if zhu < 10000 {
            return Ok(()) // ingore < 1w zhu amt
        }
        let from_addr = action
            .from
            .real(aptrs)
            .map_err(|e| db_fault(format!("hascan action address failed: {e}")))?;
        let to_addr = action
            .to
            .real(aptrs)
            .map_err(|e| db_fault(format!("hascan action address failed: {e}")))?;
        let from_aid = record_addr_id(dbtx, adrs, setting, &from_addr, blkts)?;
        let to_aid = record_addr_id(dbtx, adrs, setting, &to_addr, blkts)?;
        let mut stmt = dbtx.prepare_cached(sqlirt)?;
        stmt.insert((height, from_aid, to_aid, COINTY_ZHU, zhu))?;
        let active = record_current_active(setting, height);
        add_uint4(&mut active.trszhu, 1, "HAC transfer count")?;
        add_uint16(&mut active.mvzhu, zhu, "HAC transfer amount")?;

    /******** Satoshi ********/

    } else if kid == SatToTrs::KIND {

        let action = action_ref!(SatToTrs);
        let to_addr = real_addr!(action.to);
        let to_aid = record_addr_id(dbtx, adrs, setting, &to_addr, blkts)?;
        let sat = action.satoshi.uint();
        let mut stmt = dbtx.prepare_cached(sqlirt)?;
        stmt.insert((height, main_aid, to_aid, COINTY_SAT, sat))?;
        let active = record_current_active(setting, height);
        add_uint4(&mut active.trssat, 1, "SAT transfer count")?;
        add_uint12(&mut active.mvsat, sat, "SAT transfer amount")?;

    } else if kid == SatFromTrs::KIND {

        let action = action_ref!(SatFromTrs);
        let from_addr = real_addr!(action.from);
        let from_aid = record_addr_id(dbtx, adrs, setting, &from_addr, blkts)?;
        let sat = action.satoshi.uint();
        let mut stmt = dbtx.prepare_cached(sqlirt)?;
        stmt.insert((height, from_aid, main_aid, COINTY_SAT, sat))?;
        let active = record_current_active(setting, height);
        add_uint4(&mut active.trssat, 1, "SAT transfer count")?;
        add_uint12(&mut active.mvsat, sat, "SAT transfer amount")?;

    } else if kid == SatFromToTrs::KIND {

        let action = action_ref!(SatFromToTrs);
        let from_addr = real_addr!(action.from);
        let to_addr = real_addr!(action.to);
        let from_aid = record_addr_id(dbtx, adrs, setting, &from_addr, blkts)?;
        let to_aid = record_addr_id(dbtx, adrs, setting, &to_addr, blkts)?;
        let sat = action.satoshi.uint();
        let mut stmt = dbtx.prepare_cached(sqlirt)?;
        stmt.insert((height, from_aid, to_aid, COINTY_SAT, sat))?;
        let active = record_current_active(setting, height);
        add_uint4(&mut active.trssat, 1, "SAT transfer count")?;
        add_uint12(&mut active.mvsat, sat, "SAT transfer amount")?;
    
    /******** Diamond ********/

    } else if kid == DiaSingleTrs::KIND {

        let action = action_ref!(DiaSingleTrs);
        let to_addr = real_addr!(action.to);
        let to_aid = record_addr_id(dbtx, adrs, setting, &to_addr, blkts)?;
        let dia = 1_u64; // only one
        let mut stmt = dbtx.prepare_cached(sqlirt)?;
        stmt.insert((height, main_aid, to_aid, COINTY_DIA, dia))?;
        let active = record_current_active(setting, height);
        add_uint4(&mut active.trsdia, 1, "diamond transfer count")?;
        add_uint8(&mut active.mvdia, dia, "diamond transfer amount")?;

        // diamovedate.insert(action.diamond, blkts);


    } else if kid == DiaFromTrs::KIND {

        let action = action_ref!(DiaFromTrs);
        let from_addr = real_addr!(action.from);
        let from_aid = record_addr_id(dbtx, adrs, setting, &from_addr, blkts)?;
        let dia = action.diamonds.length() as u64;
        let mut stmt = dbtx.prepare_cached(sqlirt)?;
        stmt.insert((height, from_aid, main_aid, COINTY_DIA, dia))?;
        let active = record_current_active(setting, height);
        add_uint4(&mut active.trsdia, 1, "diamond transfer count")?;
        add_uint8(&mut active.mvdia, dia, "diamond transfer amount")?;

        // for dia in action.diamonds.list() {
        //     diamovedate.insert(*dia, blkts);
        // }

    } else if kid == DiaToTrs::KIND {

        let action = action_ref!(DiaToTrs);
        let to_addr = real_addr!(action.to);
        let to_aid = record_addr_id(dbtx, adrs, setting, &to_addr, blkts)?;
        let dia = action.diamonds.length() as u64;
        let mut stmt = dbtx.prepare_cached(sqlirt)?;
        stmt.insert((height, main_aid, to_aid, COINTY_DIA, dia))?;
        let active = record_current_active(setting, height);
        add_uint4(&mut active.trsdia, 1, "diamond transfer count")?;
        add_uint8(&mut active.mvdia, dia, "diamond transfer amount")?;

        // for dia in action.diamonds.list() {
        //     diamovedate.insert(*dia, blkts);
        // }


    } else if kid == DiaFromToTrs::KIND {

        let action = action_ref!(DiaFromToTrs);
        let from_addr = real_addr!(action.from);
        let from_aid = record_addr_id(dbtx, adrs, setting, &from_addr, blkts)?;
        let to_addr = real_addr!(action.to);
        let to_aid = record_addr_id(dbtx, adrs, setting, &to_addr, blkts)?;
        let dia = action.diamonds.length() as u64;
        let mut stmt = dbtx.prepare_cached(sqlirt)?;
        stmt.insert((height, from_aid, to_aid, COINTY_DIA, dia))?;
        let active = record_current_active(setting, height);
        add_uint4(&mut active.trsdia, 1, "diamond transfer count")?;
        add_uint8(&mut active.mvdia, dia, "diamond transfer amount")?;

        // for dia in action.diamonds.list() {
        //     diamovedate.insert(*dia, blkts);
        // }


    /******** Diamond mint ********/

    } else if kid == DiamondMint::KIND {

        let action = action_ref!(DiamondMint);
        let miner_addr = &action.d.address;
        let (_, accobj) = record_addr_as_mut(dbtx, adrs, setting, miner_addr, blkts)?;
        accobj.minted_diamond += 1;


    /******** Channel Operate ********/


    } else if kid == ChannelOpen::KIND {

        let action = action_ref!(ChannelOpen);
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

        let action = action_ref!(ChannelClose);
        let tar_id = action.channel_id.to_vec();
        let mut stmt = dbtx.prepare_cached(sqlopt)?;
        stmt.insert((height, OPTY_CH_CLOSE, main_aid, 0, tar_id, ""))?;

    }
    
    


    Ok(())

}
