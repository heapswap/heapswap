use prost_build::Config;
use std::env;
use std::fs;
use std::fs::File;
use std::io::Result;
use std::io::Write;
use std::path::{Path, PathBuf};

fn find_proto_files(dir: &Path) -> Result<Vec<PathBuf>> {
    let mut proto_files = Vec::new();
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            proto_files.extend(find_proto_files(&path)?);
        } else if path.extension().and_then(|s| s.to_str()) == Some("proto") {
            proto_files.push(path);
        }
    }
    Ok(proto_files)
}

fn main() -> Result<()> {
    let src_dir = Path::new("protos");

    let proto_files = find_proto_files(&src_dir).expect("Failed to find proto files");

    let proto_strs = proto_files
        .iter()
        .map(|path| path.to_str().unwrap())
        .collect::<Vec<_>>();

    Config::new()
        .include_file("lib.rs")
        .compile_protos(&proto_strs, &["protos/"])
        .unwrap();

    let out_dir = env::var("OUT_DIR").unwrap();

    let mut debug_file = File::create("./debug_info.txt")?;

    writeln!(debug_file, "OUT_DIR: {}", out_dir)?;

    println!("cargo:rerun-if-changed=./");
    //println!("cargo:rerun-if-changed=protos/");

    Ok(())
}
