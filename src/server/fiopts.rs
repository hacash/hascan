




defineQueryObject!{ Q1856,
    limit, Option<u64>, None,
    page, Option<u64>, None,
    both, Option<String>, None, // Address
}

async fn defi_operate(State(ctx): State<ApiCtx>, q: Query<Q1856>) -> impl IntoResponse  {
    let res = query_defi_operate(ctx, q);
    if let Err(e) = res {
        return api_error(&e.to_string())
    }
    // ok
    api_data(res.unwrap())
}



fn query_defi_operate(ctx: ApiCtx, q: Query<Q1856>) -> DBResult<JsonObject> {
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

    // relate address
    if both.len() > 0 {
        if let Err(e) = Address::from_readable(&both) {
            return errf(format!("address {} format error: {}", &both, &e))
        }
        let Some(both_aid) = query_addr_id(dbconn, &both)? else {
            return Ok(empty) // not find
        };
        adrcond = format!("aid1 = {} OR aid2 = {}", both_aid, both_aid);
    } else {
        adrcond = format!("2 > 0");
    }

    let mut datalist = Vec::new();

    // sql
    let qrsql = format!("SELECT height,kind,aid1,aid2,tarid,data FROM defi_operate WHERE {} ORDER BY height DESC LIMIT {},{}",
        adrcond, start, limit,
    );

    // println!("{}", qrsql);
    
    let mut stmt = dbconn.prepare(qrsql.as_str())?;
    let mut qres = stmt.query(())?;
    while let Some(row) = qres.next()? {
        let hei: u64 = row.get(0).unwrap();
        let kind: u8 = row.get(1).unwrap();
        // println!("row hei = {}", hei);
        let aid1: u64 = row.get(2).unwrap();
        let aid2: u64 = row.get(3).unwrap();
        let tarid: Vec<u8> = row.get(4).unwrap();
        let note: String = row.get(5).unwrap();
        addrs.insert(aid1, s!(""));
        addrs.insert(aid2, s!(""));
        // item
        datalist.push((hei, aid1, aid2, kind, tarid.hex(), note));
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