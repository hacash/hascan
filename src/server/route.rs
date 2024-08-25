
pub fn routes(mut ctx: ApiCtx) -> Router {

    let lrt = Router::new().route("/", get(console))
    
    // query paths
    .route("/query/ranking/top100", get(ranking_top100))
    .route("/query/chain/active", get(chain_active))
    .route("/query/coin/transfer", get(coin_transfer))
    .route("/query/action/operate", get(coin_transfer))
    
    ;

    // ok
    Router::new().merge(lrt).with_state(ctx)
}



async fn console(State(ctx): State<ApiCtx>, req: Request) -> impl IntoResponse {
    let mut svtips = "";
    if ctx.cnf.delaysavesetting > 0 {
        crate::save_setting(&ctx.setting.lock().unwrap()).unwrap();
        svtips = "<p>Save settings successfully!<p>"
    }

    /*/ test print
    let mut resstr = "".to_owned();
    for (dia, blkt) in ctx.diamovedate.lock().unwrap().iter() {
        resstr += format!("{},{}\n", dia.readable(), timefmt(*blkt, "%Y%m%d")).as_str();
    }
    return (HeaderMap::new(), resstr);
    */

    // render
    ( html_headers(), format!(r#"<html><head><title>Hacash Hascan console</title></head><body>
        <h3>Hacash Hascan console</h3>
        {}
        </body></html>"#,
        svtips,
    ))
}
