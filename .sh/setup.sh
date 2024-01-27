# for unstable features 
# rustup update -- nightly
# rustup default nightly

# # for building faiss
# sudo apt install libblas-dev
# sudo apt-get install liblapack-dev

# # for building rocksdb
# sudo apt-get install libgflags-dev libsnappy-dev zlib1g-dev libbz2-dev liblz4-dev libzstd-dev

sudo apt install -y pkg-config libssl-dev clang 

npm install --save-dev prettier prettier-plugin-toml

# add rust and json formatter
cat << 'EOF' > .git/hooks/pre-commit
#!/bin/bash

# Run cargo fmt on staged Rust files
rust_files=$(git diff --cached --name-only --diff-filter=ACM ".rs")
if [[ -n "$rust_files" ]]; then
    echo "Running cargo fmt on staged Rust files:"
    echo "$rust_files"
    cargo fmt -- "$rust_files"
    git add $rust_files
fi

# Run Prettier for all staged files
all_files=$(git diff --cached --name-only --diff-filter=ACM)
if [[ -n "$all_files" ]]; then
    echo "Running prettier on staged files:"
    echo "$all_files"
    prettier --write "$all_files"
    git add "$all_files"
fi

EOF