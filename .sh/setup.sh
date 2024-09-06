sudo apt install -y pkg-config libssl-dev clang lld libogg-dev cmake libclang-dev libssl-dev libudev-dev libglib2.0-dev

npm install
npm install -g prettier@2.8.8 prettier-plugin-rust@0.1.9 prettier-plugin-toml@0.3.1

cargo install cargo-watch
cargo install wasm-server-runner
cargo install pb-rs
#cargo install sccache --locked

rustup default nightly
rustup component add rustfmt
rustup target add wasm32-unknown-unknown

# setup protobufs
sudo apt-get install protobuf-compiler -y
cargo install protobuf-codegen

# install wasm-pack
curl https://rustwasm.github.io/wasm-pack/installer/init.sh -sSf | sh