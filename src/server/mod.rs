use std::collections::HashMap;
use std::sync::Arc;

use base::{ApiExecCtx, ApiRequest, ApiResponse, ApiRoute, ApiService};
use field::Address;
use rusqlite::params_from_iter;
use serde_json::{Value, json};

use crate::scaner::ScanInner;

pub struct ExplorerApi {
    inner: Arc<ScanInner>,
}

pub(crate) fn service(inner: Arc<ScanInner>) -> Arc<dyn ApiService> {
    Arc::new(ExplorerApi { inner })
}

impl ApiService for ExplorerApi {
    fn name(&self) -> &str {
        "hascan"
    }

    fn routes(&self) -> Vec<ApiRoute> {
        let route =
            |path: &str,
             inner: Arc<ScanInner>,
             handler: fn(&ScanInner, ApiRequest) -> Result<Value, String>| {
                ApiRoute::get(path, move |_ctx: &ApiExecCtx, request| {
                    response(handler(&inner, request))
                })
            };
        vec![
            route("/explorer/status", self.inner.clone(), status),
            route(
                "/explorer/query/ranking/top100",
                self.inner.clone(),
                ranking_top100,
            ),
            route(
                "/explorer/query/chain/active",
                self.inner.clone(),
                chain_active,
            ),
            route(
                "/explorer/query/coin/transfer",
                self.inner.clone(),
                coin_transfer,
            ),
            route(
                "/explorer/query/defi/operate",
                self.inner.clone(),
                defi_operate,
            ),
            route(
                "/explorer/query/address/count",
                self.inner.clone(),
                address_count,
            ),
            route(
                "/explorer/query/ecosystem/assets",
                self.inner.clone(),
                ecosystem_assets,
            ),
            route(
                "/explorer/query/ecosystem/contracts",
                self.inner.clone(),
                ecosystem_contracts,
            ),
        ]
    }
}

fn response(result: Result<Value, String>) -> ApiResponse {
    match result {
        Ok(data) => ApiResponse::json(json!({"ret": 0, "data": data}).to_string()),
        Err(error) => ApiResponse::json(json!({"ret": 1, "error": error}).to_string()),
    }
}

fn status(inner: &ScanInner, _request: ApiRequest) -> Result<Value, String> {
    let height = inner
        .setting
        .lock()
        .map_err(|_| "hascan settings lock poisoned".to_owned())?
        .height
        .uint();
    let error = inner
        .last_error
        .lock()
        .map_err(|_| "hascan error lock poisoned".to_owned())?
        .clone();
    Ok(json!({"height": height, "healthy": error.is_none(), "error": error}))
}

fn ranking_top100(inner: &ScanInner, request: ApiRequest) -> Result<Value, String> {
    let coin = request.query("coin").unwrap_or("HAC").to_ascii_uppercase();
    let setting = inner
        .setting
        .lock()
        .map_err(|_| "hascan settings lock poisoned".to_owned())?;
    let (ranking, divisor) = match coin.as_str() {
        "HAC" => (&setting.rank_zhu, 100_000_000.0),
        "BTC" => (&setting.rank_sat, 100_000_000.0),
        "HACD" => (&setting.rank_dia, 1.0),
        _ => return Err("coin must be HAC, BTC, or HACD".to_owned()),
    };
    let list: Vec<_> = ranking
        .as_list()
        .iter()
        .take(100)
        .map(|item| json!([item.addr.to_readable(), item.amount.uint() as f64 / divisor]))
        .collect();
    Ok(json!({"num": list.len(), "list": list}))
}

fn chain_active(inner: &ScanInner, _request: ApiRequest) -> Result<Value, String> {
    let setting = inner
        .setting
        .lock()
        .map_err(|_| "hascan settings lock poisoned".to_owned())?;
    let list: Vec<_> = setting
        .chain_active
        .as_list()
        .iter()
        .map(|item| {
            json!([
                item.secnum.uint(),
                item.newadr.uint(),
                item.txs.uint(),
                item.trszhu.uint(),
                item.mvzhu.uint() as f64 / 100_000_000.0,
                item.trssat.uint(),
                item.mvsat.uint() as f64 / 100_000_000.0,
                item.trsdia.uint(),
                item.mvdia.uint()
            ])
        })
        .collect();
    Ok(json!({"num": list.len(), "list": list}))
}

fn pagination(request: &ApiRequest) -> Result<(u64, u64), String> {
    let parse = |key: &str, default| match request.query(key) {
        Some(value) => value
            .parse::<u64>()
            .map_err(|_| format!("{key} must be an unsigned integer")),
        None => Ok(default),
    };
    let limit = parse("limit", 15)?;
    let page = parse("page", 1)?;
    if !(1..=200).contains(&limit) {
        return Err("limit must be between 1 and 200".to_owned());
    }
    if page == 0 {
        return Err("page must be greater than zero".to_owned());
    }
    let offset = page
        .checked_sub(1)
        .and_then(|value| value.checked_mul(limit))
        .ok_or_else(|| "page and limit are too large".to_owned())?;
    if offset > i64::MAX as u64 {
        return Err("page and limit are too large".to_owned());
    }
    Ok((limit, offset))
}

fn checked_address(value: &str) -> Result<String, String> {
    Address::from_readable(value).map_err(|e| format!("address {value} is invalid: {e}"))?;
    Ok(value.to_owned())
}

fn query_address_id(conn: &rusqlite::Connection, address: &str) -> Result<Option<u64>, String> {
    let mut stmt = conn
        .prepare_cached("SELECT id FROM account WHERE address = ?1")
        .map_err(|e| e.to_string())?;
    let mut rows = stmt.query([address]).map_err(|e| e.to_string())?;
    rows.next()
        .map_err(|e| e.to_string())?
        .map(|row| row.get(0).map_err(|e| e.to_string()))
        .transpose()
}

fn query_address_map(
    conn: &rusqlite::Connection,
    ids: impl IntoIterator<Item = u64>,
) -> Result<HashMap<u64, String>, String> {
    let ids: Vec<u64> = ids.into_iter().collect();
    if ids.is_empty() {
        return Ok(HashMap::new());
    }
    let placeholders = std::iter::repeat_n("?", ids.len())
        .collect::<Vec<_>>()
        .join(",");
    let sql = format!("SELECT id,address FROM account WHERE id IN ({placeholders})");
    let mut stmt = conn.prepare(&sql).map_err(|e| e.to_string())?;
    let mut rows = stmt
        .query(params_from_iter(ids.iter()))
        .map_err(|e| e.to_string())?;
    let mut out = HashMap::new();
    while let Some(row) = rows.next().map_err(|e| e.to_string())? {
        out.insert(
            row.get(0).map_err(|e| e.to_string())?,
            row.get(1).map_err(|e| e.to_string())?,
        );
    }
    Ok(out)
}

fn coin_transfer(inner: &ScanInner, request: ApiRequest) -> Result<Value, String> {
    let (limit, offset) = pagination(&request)?;
    let filter = [("from", "from_aid"), ("to", "to_aid"), ("both", "both")]
        .into_iter()
        .find_map(|(query, column)| {
            request
                .query(query)
                .filter(|v| !v.is_empty())
                .map(|v| (column, v))
        });
    let conn = inner
        .dbconn
        .lock()
        .map_err(|_| "hascan database lock poisoned".to_owned())?;
    let condition = match filter {
        None => "1 = 1".to_owned(),
        Some((column, address)) => {
            let address = checked_address(address)?;
            let Some(id) = query_address_id(&conn, &address)? else {
                return Ok(json!({"addrs": {}, "list": []}));
            };
            if column == "both" {
                format!("(from_aid = {id} OR to_aid = {id})")
            } else {
                format!("{column} = {id}")
            }
        }
    };
    let sql = format!(
        "SELECT height,from_aid,to_aid,coin_type,coin_amt FROM coin_transfer \
         WHERE {condition} ORDER BY height DESC,id DESC LIMIT ?1 OFFSET ?2"
    );
    let mut stmt = conn.prepare(&sql).map_err(|e| e.to_string())?;
    let mut rows = stmt.query((limit, offset)).map_err(|e| e.to_string())?;
    let mut list = Vec::new();
    let mut ids = Vec::new();
    while let Some(row) = rows.next().map_err(|e| e.to_string())? {
        let from: u64 = row.get(1).map_err(|e| e.to_string())?;
        let to: u64 = row.get(2).map_err(|e| e.to_string())?;
        ids.extend([from, to]);
        list.push(json!([
            row.get::<_, u64>(0).map_err(|e| e.to_string())?,
            from,
            to,
            row.get::<_, u8>(3).map_err(|e| e.to_string())?,
            row.get::<_, u64>(4).map_err(|e| e.to_string())?
        ]));
    }
    drop(rows);
    drop(stmt);
    Ok(json!({"addrs": query_address_map(&conn, ids)?, "list": list}))
}

fn defi_operate(inner: &ScanInner, request: ApiRequest) -> Result<Value, String> {
    let (limit, offset) = pagination(&request)?;
    let conn = inner
        .dbconn
        .lock()
        .map_err(|_| "hascan database lock poisoned".to_owned())?;
    let condition = match request.query("both").filter(|value| !value.is_empty()) {
        None => "1 = 1".to_owned(),
        Some(address) => {
            let address = checked_address(address)?;
            let Some(id) = query_address_id(&conn, &address)? else {
                return Ok(json!({"addrs": {}, "list": []}));
            };
            format!("(aid1 = {id} OR aid2 = {id})")
        }
    };
    let sql = format!(
        "SELECT height,kind,aid1,aid2,hex(tarid),data FROM defi_operate \
         WHERE {condition} ORDER BY height DESC,id DESC LIMIT ?1 OFFSET ?2"
    );
    let mut stmt = conn.prepare(&sql).map_err(|e| e.to_string())?;
    let mut rows = stmt.query((limit, offset)).map_err(|e| e.to_string())?;
    let mut list = Vec::new();
    let mut ids = Vec::new();
    while let Some(row) = rows.next().map_err(|e| e.to_string())? {
        let aid1: u64 = row.get(2).map_err(|e| e.to_string())?;
        let aid2: u64 = row.get(3).map_err(|e| e.to_string())?;
        ids.extend([aid1, aid2]);
        list.push(json!([
            row.get::<_, u64>(0).map_err(|e| e.to_string())?,
            aid1,
            aid2,
            row.get::<_, u8>(1).map_err(|e| e.to_string())?,
            row.get::<_, String>(4).map_err(|e| e.to_string())?,
            row.get::<_, String>(5).map_err(|e| e.to_string())?
        ]));
    }
    drop(rows);
    drop(stmt);
    Ok(json!({"addrs": query_address_map(&conn, ids)?, "list": list}))
}

fn address_count(inner: &ScanInner, request: ApiRequest) -> Result<Value, String> {
    let raw = request.query("address").unwrap_or("");
    let addresses: Vec<String> = raw
        .split(',')
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(checked_address)
        .collect::<Result<_, _>>()?;
    if addresses.is_empty() || addresses.len() > 200 {
        return Err("address count must be between 1 and 200".to_owned());
    }
    let conn = inner
        .dbconn
        .lock()
        .map_err(|_| "hascan database lock poisoned".to_owned())?;
    let mut stmt = conn
        .prepare_cached(
            "SELECT id,address,minted_diamond,block_reward,used_fee,timestamp \
             FROM account WHERE address = ?1",
        )
        .map_err(|e| e.to_string())?;
    let mut list = Vec::new();
    for address in addresses {
        let mut rows = stmt.query([address]).map_err(|e| e.to_string())?;
        if let Some(row) = rows.next().map_err(|e| e.to_string())? {
            list.push(json!({
                "id": row.get::<_, u64>(0).map_err(|e| e.to_string())?,
                "address": row.get::<_, String>(1).map_err(|e| e.to_string())?,
                "minted_diamond": row.get::<_, u64>(2).map_err(|e| e.to_string())?,
                "block_reward": row.get::<_, u64>(3).map_err(|e| e.to_string())?,
                "used_fee": row.get::<_, f64>(4).map_err(|e| e.to_string())?,
                "timestamp": row.get::<_, u64>(5).map_err(|e| e.to_string())?
            }));
        }
    }
    Ok(json!({"list": list}))
}

fn ecosystem_assets(inner: &ScanInner, request: ApiRequest) -> Result<Value, String> {
    let (limit, offset) = pagination(&request)?;
    let conn = inner
        .dbconn
        .lock()
        .map_err(|_| "hascan database lock poisoned".to_owned())?;
    let mut stmt = conn
        .prepare(
            "SELECT serial,height,issuer_aid,ticket,name,decimal,supply,protocol_cost \
             FROM asset_create ORDER BY height DESC,serial DESC LIMIT ?1 OFFSET ?2",
        )
        .map_err(|e| e.to_string())?;
    let mut rows = stmt.query((limit, offset)).map_err(|e| e.to_string())?;
    let mut list = Vec::new();
    let mut ids = Vec::new();
    while let Some(row) = rows.next().map_err(|e| e.to_string())? {
        let issuer_aid: u64 = row.get(2).map_err(|e| e.to_string())?;
        ids.push(issuer_aid);
        list.push(json!({
            "serial": row.get::<_, u64>(0).map_err(|e| e.to_string())?,
            "height": row.get::<_, u64>(1).map_err(|e| e.to_string())?,
            "issuer_aid": issuer_aid,
            "ticket": row.get::<_, String>(3).map_err(|e| e.to_string())?,
            "name": row.get::<_, String>(4).map_err(|e| e.to_string())?,
            "decimal": row.get::<_, u8>(5).map_err(|e| e.to_string())?,
            "supply": row.get::<_, String>(6).map_err(|e| e.to_string())?,
            "protocol_cost": row.get::<_, String>(7).map_err(|e| e.to_string())?
        }));
    }
    drop(rows);
    drop(stmt);
    Ok(json!({"addrs": query_address_map(&conn, ids)?, "list": list}))
}

fn ecosystem_contracts(inner: &ScanInner, request: ApiRequest) -> Result<Value, String> {
    let (limit, offset) = pagination(&request)?;
    let conn = inner
        .dbconn
        .lock()
        .map_err(|_| "hascan database lock poisoned".to_owned())?;
    let mut stmt = conn
        .prepare(
            "SELECT address,height,deployer_aid,nonce,code_size,protocol_cost \
             FROM contract_deploy ORDER BY height DESC,address DESC LIMIT ?1 OFFSET ?2",
        )
        .map_err(|e| e.to_string())?;
    let mut rows = stmt.query((limit, offset)).map_err(|e| e.to_string())?;
    let mut list = Vec::new();
    let mut ids = Vec::new();
    while let Some(row) = rows.next().map_err(|e| e.to_string())? {
        let deployer_aid: u64 = row.get(2).map_err(|e| e.to_string())?;
        ids.push(deployer_aid);
        list.push(json!({
            "address": row.get::<_, String>(0).map_err(|e| e.to_string())?,
            "height": row.get::<_, u64>(1).map_err(|e| e.to_string())?,
            "deployer_aid": deployer_aid,
            "nonce": row.get::<_, u32>(3).map_err(|e| e.to_string())?,
            "code_size": row.get::<_, u64>(4).map_err(|e| e.to_string())?,
            "protocol_cost": row.get::<_, String>(5).map_err(|e| e.to_string())?
        }));
    }
    drop(rows);
    drop(stmt);
    Ok(json!({"addrs": query_address_map(&conn, ids)?, "list": list}))
}

#[cfg(test)]
mod tests {
    use std::collections::{HashMap, HashSet};

    use base::Scaner;

    use super::*;
    use crate::scaner::{BlkScaner, BlkScrConfig};
    use crate::setting::ScanSettings;

    fn request(query: &[(&str, &str)]) -> ApiRequest {
        ApiRequest {
            query: query
                .iter()
                .map(|(key, value)| ((*key).to_owned(), (*value).to_owned()))
                .collect::<HashMap<_, _>>(),
            ..Default::default()
        }
    }

    #[test]
    fn pagination_validates_bounds_and_input() {
        assert_eq!(pagination(&request(&[])).unwrap(), (15, 0));
        assert_eq!(
            pagination(&request(&[("limit", "200"), ("page", "3")])).unwrap(),
            (200, 400)
        );
        assert!(pagination(&request(&[("limit", "201")])).is_err());
        assert!(pagination(&request(&[("page", "0")])).is_err());
        assert!(pagination(&request(&[("limit", "nope")])).is_err());
        assert!(pagination(&request(&[("page", &u64::MAX.to_string())])).is_err());
    }

    #[test]
    fn explorer_routes_are_unique_and_prefixed() {
        let scaner = BlkScaner::new(
            BlkScrConfig::default(),
            ScanSettings::default(),
            rusqlite::Connection::open_in_memory().unwrap(),
        )
        .unwrap();
        let routes = scaner.api_services().remove(0).routes();
        let paths: HashSet<_> = routes.iter().map(|route| route.path.as_str()).collect();
        assert_eq!(paths.len(), routes.len());
        assert!(paths.iter().all(|path| path.starts_with("/explorer/")));
    }
}
