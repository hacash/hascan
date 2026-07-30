# hascan

`hascan` is the SQLite-backed explorer indexer for the Rust Hacash fullnode. It
runs in the same process as the node, consumes stable blocks through
`base::Scaner`, and publishes explorer routes through the node's HTTP server.

## Build

The repository is expected next to the current fullnode checkout:

```text
rust/
|-- fullnode/
`-- hascan/
```

Install the SQLite development package and build with the default Sled chain
database backend:

```sh
sudo apt-get install libsqlite3-dev pkg-config
cargo build --release
cp hascan.config.example.ini hascan.config.ini
./target/release/hascan /absolute/path/to/hascan.config.ini
```

The first argument is the shared node/indexer INI file and defaults to
`hascan.config.ini`. Relative paths are resolved from the executable directory,
so an absolute path is recommended.

Select exactly one fullnode chain database backend when disabling defaults:

```sh
cargo build --release --no-default-features --features db-sled
cargo build --release --no-default-features --features db-rusty-leveldb
cargo build --release --no-default-features --features db-leveldb-sys
cargo build --release --no-default-features --features db-rocksdb
```

SQLite remains the explorer index database for every chain backend.

## Configuration

Copy the tracked `hascan.config.example.ini` and adjust it for the deployment.
The file contains normal fullnode sections plus `[hascan]`:

```ini
[engine]
data_dir = ./hacash_mainnet_data
fast_sync = true

[p2p]
listen_ip = 0.0.0.0
listen_port = 13317
find_nodes = false
boot_nodes = 127.0.0.1:33311

[server]
listen_ip = 127.0.0.1
listen_port = 18081
debug_routes = false

[mint]
chain_id = 0
diamond_form = true

[miner]
enable = false

[diamond_miner]
enable = false

[hascan]
datadir = ./hacash_scan_data
synchronous = NORMAL
```

`[hascan].datadir` stores `database.db3` and the compatibility mirror
`settings.dat`. `synchronous` accepts SQLite's `OFF`, `NORMAL`, `FULL`, or
`EXTRA` modes. There is no independent hascan listener: all explorer endpoints
use `[server].listen_ip` and `[server].listen_port`.

Do not point two hascan processes at the same explorer directory. Keep the
chain and explorer directories on durable storage, and back up both when a
consistent operational snapshot is required.

## Explorer API

The integrated GET routes are:

| Route | Main query parameters |
| --- | --- |
| `/explorer/status` | none |
| `/explorer/query/ranking/top100` | `coin=HAC|BTC|HACD` |
| `/explorer/query/chain/active` | none |
| `/explorer/query/coin/transfer` | `page`, `limit`, `from`, `to`, or `both` |
| `/explorer/query/defi/operate` | `page`, `limit`, `both` |
| `/explorer/query/address/count` | comma-separated `address` |

`limit` is capped at 200. Responses preserve the existing explorer envelope:
`{"ret":0,"data":...}` on success and `{"ret":1,"error":"..."}` on a query
error. `/explorer/status` reports the durable checkpoint and any background
indexing error.

## Architecture and Lifecycle

```text
stable block -> ChainListener -> Scaner::on_block -> coalesced worker wake-up
                                           |
                                           v
chain history/state <- ScanerView <- checkpoint catch-up -> SQLite transaction
                                                          -> ranking snapshot

hascan ApiService -> fullnode service registry -> shared HTTP server
```

Startup opens the fullnode and hascan databases, attaches the chain listener,
starts the index worker, and synchronously catches up from the durable hascan
checkpoint before P2P and HTTP listeners open. Each indexed block commits its
transfer/DeFi rows, account changes, checkpoint, and serialized settings in one
SQLite transaction. Live notifications do no database work on the chain thread;
the worker catches up from stable history instead.

Ranking balances are read in batches from one validated stable-state snapshot.
The scanner currently records the same top-level Action set as the legacy
hascan implementation; nested Action traversal is outside this integration.

## Legacy Database Migration

On first startup with an older `hacash_scan_data` directory, hascan creates the
`scan_status` table without rebuilding existing explorer tables. It uses the
old `settings.dat` height when present; if that height is zero, it infers a
checkpoint from the highest transfer or DeFi row and prints a warning. The
existing database is not deleted or rewritten wholesale.

Back up `hacash_scan_data` before the first migrated run. The inferred height is
necessarily conservative because blocks with no indexed top-level actions
leave no legacy row. Rebuild into a new explorer directory when exact historical
completeness is required.

## Development

Run the focused checks before changing the scanner contract or persistence
format:

```sh
cargo fmt --check
cargo test --all-targets
cargo clippy --all-targets -- -W clippy::all
```

The integration points are `base::Scaner`, `base::ScanerView`,
`base::ApiService`, and `app::Fullnode::open(path, Some(scaner))`. New explorer
features should remain in hascan and use those narrow interfaces rather than
depending on concrete chain-engine internals.
