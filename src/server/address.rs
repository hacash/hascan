




defineQueryObject!{ Q5363,
    address, String, s!(""), // address list
}

async fn address_count(State(ctx): State<ApiCtx>, q: Query<Q5363>) -> impl IntoResponse  {
    let res = query_address_count(ctx, q);
    if let Err(e) = res {
        return api_error(&e.to_string())
    }
    // ok
    api_data(res.unwrap())
}



fn query_address_count(ctx: ApiCtx, q: Query<Q5363>) -> DBResult<JsonObject> {
    // q.address

    let errf = |e: String| {
        Err(rusqlite::Error::InvalidParameterName(e))
    };

    let addrs: Vec<String> = q.address.replace(" ", "").trim_matches(',')
        .split(',').map(|s|s.to_string()).collect();
    if addrs.len() > 200 {
        return errf(s!("address count cannot more than 200"))
    }
    let mut addrids = String::new();
    for a in &addrs {
        if let Err(e) = Address::from_readable(a) {
            return errf(format!("address {} format error", a))
        }
        addrids += &format!("\"{}\",", &a);
        // check next
    }
    let addrids = addrids.trim_matches(',');
    if addrids.len() <= 0 {
        return errf(s!("address must give"))
    }

    // sql
    let qrsql = format!("SELECT minted_diamond,block_reward,used_fee,timestamp FROM account WHERE address IN({})", addrids);

    // load
    let mut datalist = Vec::with_capacity(addrs.len()); 
    let mut dbconn = ctx.dbconn.lock().unwrap();
    let mut stmt = dbconn.prepare(qrsql.as_str())?;
    let mut qres = stmt.query(())?;
    while let Some(row) = qres.next()? {
        let minted_diamond: u64 = row.get(0).unwrap();
        let block_reward: u64 = row.get(1).unwrap();
        let used_fee: f64 = row.get(2).unwrap();
        let timestamp: u64 = row.get(3).unwrap();
        datalist.push(jsondata!{
            "minted_diamond", minted_diamond,
            "block_reward", block_reward,
            "used_fee", used_fee,
            "timestamp", timestamp,
        });
    }

    let mut data = jsondata!{
        "list", datalist,
    };

    // ok
    Ok(data)
}