




defineQueryObject!{ Q8237,
    __nnn__, Option<String>, None, // HAC / BTC / HACD
}

async fn chain_active(State(ctx): State<ApiCtx>, q: Query<Q8237>) -> impl IntoResponse  {
    // q_must!(q, coin, s!("HAC"));

    let bding = ctx.setting.lock().unwrap();
    let rtlist = bding.chain_active.list();

    // deal data
    let ll = rtlist.len();
    let mut datalist = Vec::with_capacity(ll);
    for a in rtlist {
        datalist.push((
            a.secnum.uint(),
            a.newadr.uint(), // new address
            a.txs.uint(),
            a.trszhu.uint(),
            a.trssat.uint(),
            a.trsdia.uint(),
            a.mvzhu.uint() as f64 / 100000000.0, // mei
            a.mvsat.uint() as f64 / 100000000.0, // btc
            a.mvdia.uint(), // DIAMOND
        ));
    }

    let mut data = jsondata!{
        "num", ll,
        "list", datalist,
    };
    // ok
    api_data(data)



    


    
}