# hascan
Hacash block scaner for explorer


```sh
# step1: 
sudo apt-get install libsqlite3-dev 

# build
RUSTFLAGS="$RUSTFLAGS -Awarnings" cargo build && cp ./target/debug/hascan ./ && ./hascan
rm -rf hacash_*_data/

# db-level-sys
cargo build --release --no-default-features --features "db-leveldb-sys"


# release
RUSTFLAGS="$RUSTFLAGS -Awarnings" cargo build --release && cp ./target/release/hascan ./ && cp ./hascan ../fullnode/hascan_2026032301


```



