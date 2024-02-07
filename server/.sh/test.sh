#export ROCKSDB_LIB_DIR=/usr/local/lib
cargo watch \
	-i "target/*" \
	-i "dist/*" \
	-i "data/*" \
	-i "src/data/*" \
	-i "src/schemas/*" \
	-c \
	-x "test -- --nocapture"