
pub fn routes(ctx: ApiCtx) -> Router {

    let lrt = Router::new().route("/", get(console))
    
    // query paths
    .route("/query/ranking/top100", get(ranking_top100))
    .route("/query/chain/active", get(chain_active))
    .route("/query/coin/transfer", get(coin_transfer))
    .route("/query/defi/operate", get(defi_operate))
    .route("/query/address/count", get(address_count))
    
    ;

    // ok
    Router::new().merge(lrt).with_state(ctx)
}



async fn console(State(ctx): State<ApiCtx>, _req: Request) -> impl IntoResponse {
    let mut svtips = String::new();
    if ctx.cnf.delaysavesetting > 0 {
        match crate::save_setting(&ctx.cnf.data_dir, &ctx.setting.lock().unwrap()) {
            Ok(_) => {
                svtips = "<p>Save settings successfully!<p>".to_string();
            }
            Err(e) => {
                svtips = format!("<p>Save settings failed: {}<p>", e);
            }
        }
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
