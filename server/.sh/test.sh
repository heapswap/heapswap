#export ROCKSDB_LIB_DIR=/usr/local/lib
cargo watch -i "target/*" -i "dist/*" -i "data/*" -c -x "test -- --nocapture"