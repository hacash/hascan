# hascan
Hacash block scaner for explorer


```sh
# step1: 
sudo apt-get install libsqlite3-dev 

# db-level-sys
RUSTFLAGS="$RUSTFLAGS -Awarnings" cargo build && cp ./target/debug/hascan ./ && ./hascan
rm -rf hacash_*_data/

# release
RUSTFLAGS="$RUSTFLAGS -Awarnings" cargo build --release && cp ./target/release/hascan ./ && cp ./hascan ../fullnode/


```



