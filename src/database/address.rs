



/**
* return: is new, addr id
*/


pub fn record_addr_id(dbtx: &mut DBTransaction, adrs: &mut AddressCache, setting: &mut ScanSettings, 
    adrobj: &Address, blkts: u64) -> DBResult<u64> {
        let (_, aid) = record_addr_id_ex(dbtx, adrs, setting, adrobj, blkts)?;
        Ok(aid as u64)
}

pub fn record_addr_id_ex(dbtx: &mut DBTransaction, adrs: &mut AddressCache, setting: &mut ScanSettings, 
    adrobj: &Address, blkts: u64) -> DBResult<(bool, i64)> {
    let address = adrobj.readable();
    let new = true;
    let old = false;
    if let Some(adr) = adrs.get(&address) {
        return Ok((old, adr.id)) // from cache
    }
    // query from db
    let mut stmt = dbtx.prepare_cached("SELECT id,minted_diamond,block_reward,used_fee FROM account WHERE address = ?1")?;
    while let Some(row) = stmt.query([&address])?.next()? {
        let aid: i64 = row.get(0)?;
        let asto = AddressSto {
            id: aid,
            minted_diamond: row.get(1)?,
            block_reward: row.get(2)?,
            used_fee: row.get(3)?,
            timestamp: 0, // mean update not insert
        };
        adrs.insert(address, asto); // cache
        return Ok((old, aid)) // from cache
    }
    // create new account
    setting.auto_inc_address_id += 1;
    let aaid = setting.auto_inc_address_id.uint() as i64;
    let asto = AddressSto {
        id: aaid,
        timestamp: blkts,
        ..Default::default()
    };
    adrs.insert(address, asto); // cache
    Ok((new, aaid)) // new create address

    /* insert to database
    let mut stmt_irt = conn.prepare_cached("INSERT INTO account (address,timestamp) VALUES (?1, ?2)")?;
    let aid = stmt.insert((&address, blkts))?;
    let mut asto = AddressSto::new(aid);
    adrs.insert(address, asto); // cache
    Ok((new, aid)) // new create address
    */

}


/**
*
*/
pub fn insert_update_addr(dbtx: &mut DBTransaction, adrs: &AddressCache) -> DBResult<()> {

    // insert
    let mut stmt_insert = dbtx.prepare_cached("INSERT INTO account 
        (id,address,minted_diamond,block_reward,used_fee,timestamp) VALUES 
        (?1, ?2, ?3, ?4, ?5, ?6)")?;
    let mut stmt_update = dbtx.prepare_cached("UPDATE account SET 
        minted_diamond = ?1, block_reward = ?2, used_fee = ?3 WHERE id = ?4")?;
    
    // loop
    for (a, adr) in adrs {
        if adr.timestamp > 0 {
            let address = a;
            stmt_insert.insert(( adr.id, address, adr.minted_diamond, 
                adr.block_reward, adr.used_fee, adr.timestamp))?;
        } else {
            stmt_update.execute(( adr.minted_diamond, 
                adr.block_reward, adr.used_fee, adr.id ))?;
        }
    }

    Ok(())
}