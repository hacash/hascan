


// create tables
pub fn create_tables(conn: &mut Connection) -> DBResult<()> {
    let mut tx = conn.transaction()?;

    
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


    /* operate_action */


    tx.execute(
        "CREATE TABLE IF NOT EXISTS `operate_action` (
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
        "CREATE INDEX IF NOT EXISTS acc2_id on operate_action (aid1, aid2)", ()
    )?;


    
    tx.commit()
}
