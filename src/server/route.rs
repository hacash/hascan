
pub fn routes(mut ctx: ApiCtx) -> Router {

    let lrt = Router::new().route("/", get(console))
    
    // route paths
    .route("/ranking/top100", get(ranking_top100))
    .route("/chain/active", get(chain_active))
    ;

    // ok
    Router::new().merge(lrt).with_state(ctx)
}



async fn console(State(ctx): State<ApiCtx>, req: Request) -> impl IntoResponse {
    // render
    format!(r#"<html><head><title>Hacash Hascan console</title></head><body>
        <h3>Hacash Hascan console</h3>
        </body></html>"#
    )
}
