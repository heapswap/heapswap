extern crate capnpc;
use super::walk_schema_dir::walk_schema_dir;
use std::fs;
use walkdir::WalkDir;

// this function builds the capnp command
pub fn build_capnp(input_schema_folder: &str, output_schema_folder: &str) {
  
  // delete the output folder if it exists
  if fs::metadata(output_schema_folder).is_ok() {
    fs::remove_dir_all(output_schema_folder).unwrap();
  }
  
  // recreate the output folder
  fs::create_dir_all(output_schema_folder).unwrap();

  // create the capnp command
  let mut command = capnpc::CompilerCommand::new();
  command.output_path(output_schema_folder);
  command.src_prefix(input_schema_folder);

  for entry in WalkDir::new(input_schema_folder) {
    let entry = entry.unwrap();
    if entry.file_type().is_file() && entry.path().extension().unwrap() == "capnp" {
      command.file(entry.path());
    }
  }

  command.run().unwrap();

  walk_schema_dir(input_schema_folder, output_schema_folder);
}
