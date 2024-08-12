# .js and .toml
npx prettier --write .
# .rs
cargo fmt
# .proto
cd crates/subfield_proto/protos && buf format -w && cd -