


fn query_addr_id(dbconn: &mut Connection, address: &String) -> DBResult<Option<u64>> {
    let mut stmt = dbconn.prepare_cached("SELECT id FROM account WHERE address = ?1")?;
    while let Some(row) = stmt.query([address])?.next()? {
        return Ok(Some(row.get(0).unwrap()));
    }
    // not find
    Ok(None)
}



fn query_addr_maps(dbconn: &mut Connection, addrs: &mut HashMap<u64, String>) -> DBResult<()> {
    let ids = addrs.iter().map(|(k,_)|k.to_string()).collect::<Vec<String>>().join(",");
    if ids.len() == 0 {
        return Ok(()) // empty
    }
    let qrsql = format!("SELECT id,address FROM account WHERE id IN( {} )", ids);
    let mut stmt = dbconn.prepare(qrsql.as_str())?;
    let mut qres = stmt.query(())?;
    while let Some(row) = qres.next()? {
        let id: u64 = row.get(0).unwrap();
        let addr: String = row.get(1).unwrap();
        addrs.insert(id, addr);
    }
    // ok
    Ok(())
}