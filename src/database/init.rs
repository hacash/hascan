


// create tables
pub fn create_tables(conn: &mut Connection) -> DBResult<()> {
    let tx = conn.transaction()?;

    /* account */

    tx.execute(
        "CREATE TABLE IF NOT EXISTS `account` (
            id                INTEGER PRIMARY KEY AUTOINCREMENT,
            address           TEXT NOT NULL,
            minted_diamond    INTEGER NOT NULL,
            block_reward      INTEGER NOT NULL,
            used_fee          REAL NOT NULL,
            timestamp         INTEGER NOT NULL
        )", () // hac unit: mei
    )?;

    tx.execute(
        "CREATE UNIQUE INDEX IF NOT EXISTS address on account (address)", ()
    )?;


    /* coin_transfer */


    tx.execute(
        "CREATE TABLE IF NOT EXISTS `coin_transfer` (
            id                INTEGER PRIMARY KEY AUTOINCREMENT,
            height            INTEGER NOT NULL,
            from_aid          INTEGER NOT NULL,
            to_aid            INTEGER NOT NULL,
            coin_type         INTEGER NOT NULL,
            coin_amt          INTEGER NOT NULL
        )", ()
    )?;

    tx.execute(
        "CREATE INDEX IF NOT EXISTS acc_id on coin_transfer (from_aid, to_aid)", ()
    )?;


    /* defi_operate */


    tx.execute(
        "CREATE TABLE IF NOT EXISTS `defi_operate` (
            id                INTEGER PRIMARY KEY AUTOINCREMENT,
            height            INTEGER NOT NULL,
            kind              INTEGER NOT NULL,
            aid1              INTEGER NOT NULL,
            aid2              INTEGER NOT NULL,
            tarid             BLOB NOT NULL,
            data              TEXT NOT NULL
        )", ()
    )?;

    tx.execute(
        "CREATE INDEX IF NOT EXISTS acc2_id on defi_operate (aid1, aid2)", ()
    )?;

    /* Istanbul protocol event indexes */

    tx.execute(
        "CREATE TABLE IF NOT EXISTS `asset_create` (
            serial            INTEGER PRIMARY KEY,
            height            INTEGER NOT NULL,
            issuer_aid        INTEGER NOT NULL,
            ticket            TEXT NOT NULL,
            name              TEXT NOT NULL,
            decimal           INTEGER NOT NULL,
            supply            TEXT NOT NULL,
            protocol_cost     TEXT NOT NULL
        )", ()
    )?;
    tx.execute(
        "CREATE INDEX IF NOT EXISTS asset_create_height on asset_create (height DESC, serial DESC)", ()
    )?;

    tx.execute(
        "CREATE TABLE IF NOT EXISTS `contract_deploy` (
            address           TEXT PRIMARY KEY,
            height            INTEGER NOT NULL,
            deployer_aid      INTEGER NOT NULL,
            nonce             INTEGER NOT NULL,
            code_size         INTEGER NOT NULL,
            protocol_cost     TEXT NOT NULL
        )", ()
    )?;
    tx.execute(
        "CREATE INDEX IF NOT EXISTS contract_deploy_height on contract_deploy (height DESC, address DESC)", ()
    )?;

    tx.execute(
        "CREATE TABLE IF NOT EXISTS `contract_update` (
            id                INTEGER PRIMARY KEY AUTOINCREMENT,
            address           TEXT NOT NULL,
            height            INTEGER NOT NULL,
            updater_aid       INTEGER NOT NULL,
            edit_size          INTEGER NOT NULL,
            protocol_cost     TEXT NOT NULL
        )", ()
    )?;
    tx.execute(
        "CREATE INDEX IF NOT EXISTS contract_update_height on contract_update (height DESC, id DESC)", ()
    )?;

    /* scan_status */
    tx.execute(
        "CREATE TABLE IF NOT EXISTS `scan_status` (
            k                 TEXT PRIMARY KEY,
            v                 INTEGER NOT NULL
        )", ()
    )?;


    
    tx.commit()
}

const SCAN_STATUS_HEIGHT_KEY: &str = "height";
const SCAN_STATUS_SETTINGS_KEY: &str = "settings";

pub fn load_scan_height(conn: &Connection) -> DBResult<u64> {
    let mut stmt = conn.prepare_cached("SELECT v FROM scan_status WHERE k = ?1")?;
    if let Some(row) = stmt.query([SCAN_STATUS_HEIGHT_KEY])?.next()? {
        return row.get(0)
    }
    Ok(0)
}

pub fn save_scan_height_conn(conn: &Connection, height: u64) -> DBResult<()> {
    conn.execute(
        "INSERT INTO scan_status (k, v) VALUES (?1, ?2)
         ON CONFLICT(k) DO UPDATE SET v = excluded.v",
        (SCAN_STATUS_HEIGHT_KEY, height),
    )?;
    Ok(())
}

pub fn save_scan_height_tx(tx: &mut DBTransaction, height: u64) -> DBResult<()> {
    tx.execute(
        "INSERT INTO scan_status (k, v) VALUES (?1, ?2)
         ON CONFLICT(k) DO UPDATE SET v = excluded.v",
        (SCAN_STATUS_HEIGHT_KEY, height),
    )?;
    Ok(())
}

pub fn load_scan_settings(conn: &Connection) -> DBResult<Option<Vec<u8>>> {
    let mut stmt = conn.prepare_cached("SELECT v FROM scan_status WHERE k = ?1")?;
    let mut rows = stmt.query([SCAN_STATUS_SETTINGS_KEY])?;
    match rows.next()? {
        Some(row) => row.get(0).map(Some),
        None => Ok(None),
    }
}

pub fn save_scan_settings_tx(tx: &mut DBTransaction, data: &[u8]) -> DBResult<()> {
    tx.execute(
        "INSERT INTO scan_status (k, v) VALUES (?1, ?2)
         ON CONFLICT(k) DO UPDATE SET v = excluded.v",
        (SCAN_STATUS_SETTINGS_KEY, data),
    )?;
    Ok(())
}

pub fn save_scan_settings_conn(conn: &mut Connection, height: u64, data: &[u8]) -> DBResult<()> {
    let mut tx = conn.transaction()?;
    save_scan_height_tx(&mut tx, height)?;
    save_scan_settings_tx(&mut tx, data)?;
    tx.commit()
}

pub fn load_max_account_id(conn: &Connection) -> DBResult<u64> {
    let mut stmt = conn.prepare_cached("SELECT IFNULL(MAX(id), 0) FROM account")?;
    stmt.query_row((), |row| row.get(0))
}

pub fn infer_legacy_scan_height(conn: &Connection) -> DBResult<u64> {
    conn.query_row(
        "SELECT MAX(height) FROM (
             SELECT IFNULL(MAX(height), 0) AS height FROM coin_transfer
             UNION ALL
             SELECT IFNULL(MAX(height), 0) AS height FROM defi_operate
         )",
        (),
        |row| row.get(0),
    )
}

pub fn load_all_account_addresses(conn: &Connection) -> DBResult<Vec<Address>> {
    let mut stmt = conn.prepare("SELECT address FROM account ORDER BY id")?;
    let rows = stmt.query_map((), |row| row.get::<_, String>(0))?;
    let mut out = Vec::new();
    for row in rows {
        let readable = row?;
        let address = Address::from_readable(&readable).map_err(|e| {
            rusqlite::Error::FromSqlConversionFailure(
                0,
                rusqlite::types::Type::Text,
                Box::new(std::io::Error::new(std::io::ErrorKind::InvalidData, e.to_string())),
            )
        })?;
        out.push(address);
    }
    Ok(out)
}
