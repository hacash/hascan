


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

pub fn load_max_account_id(conn: &Connection) -> DBResult<u64> {
    let mut stmt = conn.prepare_cached("SELECT IFNULL(MAX(id), 0) FROM account")?;
    stmt.query_row((), |row| row.get(0))
}
