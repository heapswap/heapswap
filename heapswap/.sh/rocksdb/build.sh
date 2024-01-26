# see https://github.com/facebook/rocksdb/blob/main/INSTALL.md
make static_lib -j 8

cp librocksdb.a /usr/local/lib/librocksdb.a