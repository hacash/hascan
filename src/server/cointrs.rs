




defineQueryObject!{ Q4396,
    limit, Option<u64>, None,
    page, Option<u64>, None,
    from, Option<String>, None, // Address
    to, Option<String>, None, // Address
    both, Option<String>, None, // Address
}

async fn coin_transfer(State(ctx): State<ApiCtx>, q: Query<Q4396>) -> impl IntoResponse  {
    let res = query_coin_transfer(ctx, q);
    if let Err(e) = res {
        return api_error(&e.to_string())
    }
    // ok
    api_data(res.unwrap())
}



fn query_coin_transfer(ctx: ApiCtx, q: Query<Q4396>) -> DBResult<JsonObject> {
    q_must!(q, from, s!(""));
    q_must!(q, to, s!(""));
    q_must!(q, both, s!(""));
    q_must!(q, limit, 15);
    q_must!(q, page, 1);
    let start: u64 = (page - 1) * limit;

    let mut addrs: HashMap<u64, String> = HashMap::new();

    let mut dbconn = ctx.dbconn.lock().unwrap();
    let dbconn = &mut dbconn;

    // empty
    let empty = jsondata!{
        "addrs", addrs,
        "list", (),
    };

    let mut adrcond = "".to_owned();

    let errf = |e: String| {
        Err(rusqlite::Error::InvalidParameterName(e))
    };

    // from
    if from.len() > 0 {
        if let Err(e) = Address::from_readable(&from) {
            return errf(format!("address {} format error: {}", &from, &e))
        }
        let Some(from_aid) = query_addr_id(dbconn, &from)? else {
            return Ok(empty) // not find
        };
        adrcond = format!("from_aid = {}", from_aid);

    // to
    } else if to.len() > 0 {
        if let Err(e) = Address::from_readable(&to) {
            return errf(format!("address {} format error: {}", &from, &e))
        }
        let Some(to_aid) = query_addr_id(dbconn, &to)? else {
            return Ok(empty) // not find
        };
        adrcond = format!("to_aid = {}", to_aid);

    // both
    } else if both.len() > 0 {
        if let Err(e) = Address::from_readable(&both) {
            return errf(format!("address {} format error: {}", &from, &e))
        }
        let Some(both_aid) = query_addr_id(dbconn, &both)? else {
            return Ok(empty) // not find
        };
        adrcond = format!("from_aid = {} OR to_aid = {}", both_aid, both_aid);
    } else {
        adrcond = format!("2 > 0");

    }

    let mut datalist = Vec::new();

    // sql
    let qrsql = format!("SELECT height,from_aid,to_aid,coin_type,coin_amt FROM coin_transfer WHERE {} ORDER BY height DESC LIMIT {},{}",
        adrcond, start, limit,
    );

    // println!("{}", qrsql);
    
    let mut stmt = dbconn.prepare(qrsql.as_str())?;
    let mut qres = stmt.query(())?;
    while let Some(row) = qres.next()? {
        let hei: u64 = row.get(0).unwrap();
        // println!("row hei = {}", hei);
        let faid: u64 = row.get(1).unwrap();
        let taid: u64 = row.get(2).unwrap();
        let cty: u8 = row.get(3).unwrap();
        let amt: u64 = row.get(4).unwrap();
        addrs.insert(faid, s!(""));
        addrs.insert(taid, s!(""));
        // item
        datalist.push((hei, faid, taid, cty, amt));
    }
    drop(qres);
    drop(stmt);

    // println!("{} {}", datalist.len(), addrs.len());

    // query address
    query_addr_maps(dbconn, &mut addrs)?;

    // println!("query_addr_maps {}", addrs.len());

    let mut data = jsondata!{
        "addrs", addrs,
        "list", datalist,
    };
    // ok
    Ok(data)
}