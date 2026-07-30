fn main() -> Rerr {
    use std::path::PathBuf;
    use std::sync::Arc;

    let config_path = std::env::args()
        .nth(1)
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("hascan.config.ini"));
    let ini = sys::load_config(&config_path)?;
    let scaner = Arc::new(hascan::open(&ini)?);
    app::Fullnode::open(&config_path, Some(scaner))?.run()
}

use sys::Rerr;
