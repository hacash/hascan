



/**
* return: is new, addr id
*/
pub fn record_addr_id(adrs: &mut AddressCache, conn: &mut Connection, address: String, blkts: u64) -> Result<(bool, i64)> {
    let new = true;
    let old = false;
    if let Some(adr) = adrs.get(&address) {
        return Ok((old, adr.id)) // from cache
    }
    // query from db
    let mut stmt = conn.prepare_cached("SELECT id,minted_diamond,block_reward,used_fee FROM account WHERE address = ?1")?;
    while let Some(row) = stmt.query([&address])?.next()? {
        let aid: i64 = row.get(0)?;
        let asto = AddressSto {
            id: aid,
            minted_diamond: row.get(1)?,
            block_reward: row.get(2)?,
            used_fee: row.get(3)?,
        };
        adrs.insert(address, asto); // cache
        return Ok((old, aid)) // from cache
    }
    // create database
    let mut stmt_irt = conn.prepare_cached("INSERT INTO account (address,timestamp) VALUES (?1, ?2)")?;
    let aid = stmt.insert((&address, blkts))?;
    let mut asto = AddressSto::new(aid);
    adrs.insert(address, asto); // cache
    Ok((new, aid)) // new create address

}