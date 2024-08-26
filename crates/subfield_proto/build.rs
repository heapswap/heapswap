#![allow(unused)]
// use poem_grpc_build::compile_protos;
use prost_build::Config;
use std::env;
use std::fs;
use std::fs::File;
use std::io::{self, Write};
use std::path::{Path, PathBuf};

fn find_proto_files(dir: &Path) -> io::Result<Vec<PathBuf>> {
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

fn main() -> io::Result<()> {
	let src_dir = Path::new("proto");
	let proto_files = find_proto_files(&src_dir)?;

	let proto_strs = proto_files
		.iter()
		.map(|path| path.to_str().expect("Invalid UTF-8 in path"))
		.collect::<Vec<_>>();

	// let proto_strs: Vec<&str> = Vec::new();

	
	let mut config = Config::new();
	config
		.include_file("lib.rs")
		.compile_well_known_types()
		.compile_protos(&proto_strs, &["proto/"])?;
	
	println!("{:?}", proto_strs);
	// compile_protos(&proto_strs, &["proto/"])?;
	// compile_protos(&["helloworld.proto"], &["proto/"])?;

	let out_dir =
	 	env::var("OUT_DIR").expect("OUT_DIR environment variable not set");

	let mut debug_file = File::create("./debug_info.txt")?;
	writeln!(debug_file, "OUT_DIR: {}", out_dir)?;

	println!("cargo:rerun-if-changed=./");

	Ok(())
}

