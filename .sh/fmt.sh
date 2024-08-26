# .js and .toml
npx prettier -i --write .
# .rs
cargo fmt
# .proto
cd crates/subfield_proto/proto && buf format -w && cd -