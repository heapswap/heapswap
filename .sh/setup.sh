sudo apt install -y pkg-config libssl-dev clang lld lib libogg-dev

npm install
npm install -g prettier@2.8.8 prettier-plugin-rust@0.1.9 prettier-plugin-toml@0.3.1

cargo install cargo-watch
cargo install sccache --locked

rustup default nightly
rustup component add rustfmt
rustup target add wasm32-unknown-unknown

# setup protobufs
sudo apt-get install protobuf-compiler -y
cargo install protobuf-codegen