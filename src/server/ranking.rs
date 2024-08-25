




defineQueryObject!{ Q8364,
    coin, Option<String>, None, // HAC / BTC / HACD
}

async fn ranking_top100(State(ctx): State<ApiCtx>, q: Query<Q8364>) -> impl IntoResponse  {
    q_must!(q, coin, s!("HAC"));


    let stobj = ctx.setting.lock().unwrap();
    let rtlist = match coin.as_str() {
        "HAC"  => &stobj.rank_zhu,
        "BTC"  => &stobj.rank_sat,
        "HACD" => &stobj.rank_dia,
        _ => return api_error("param coin error"),
    }.list();

    let div: f64 = match coin.as_str() {
        "HAC"  => 100000000.0,
        "BTC"  => 100000000.0,
        _ => 1.0,
    };

    // deal data
    let ll = rtlist.len();
    let mut datalist = Vec::with_capacity(ll);
    for i in 0..ll {
        if i >= 100 {
            break // end max 100
        }
        let a = &rtlist[i];
        datalist.push((a.addr.readable(), a.amount.uint() as f64 / div));
    }

    let mut data = jsondata!{
        "num", ll,
        "list", datalist,
    };
    // ok
    api_data(data)



    


    
}