use std::fs;
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

use filetime::FileTime;
use walkdir::WalkDir;

mod build_capnp;
use crate::build_capnp::build_capnp_func;

// custom build script
fn main() {
  
  println!("Building schemas");
  
  // build the capnp files if they have changed since the last build  
  let input_schema_folder = "schemas";
  let output_schema_folder = "src/schemas";
  
  // get the last build time
  let last_build_time = get_and_update_last_build_time();
  
  // check if the capnp files have changed since the last build and rebuild them if necessary
  check_capnp(input_schema_folder, output_schema_folder, last_build_time);

  println!("Finished building schemas");
}

// get the last build time
fn get_and_update_last_build_time() -> u64 {
    // create a file to store the last build time if it doesn't exist
    let tmp_build_file = Path::new("/tmp/LAST_BUILD_TIME");

    // get the last build time
    let last_build_time = fs::read_to_string(tmp_build_file)
        .unwrap_or_else(|_| {
            fs::write(tmp_build_file, "0").expect("Unable to write file");
            String::from("0")
        })
        .parse::<u64>()
        .unwrap_or(0);

    // update the build time
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();
    fs::write(tmp_build_file, now.to_string().as_bytes()).unwrap();

    return last_build_time;
}

// check if the capnp files have changed since the last build and rebuild them if necessary
fn check_capnp(input_schema_folder: &str, output_schema_folder: &str, last_build_time: u64){
  
  let mut schema_rebuild_required = false;

  for entry in WalkDir::new(input_schema_folder) {
    let entry = entry.unwrap();
    if entry.file_type().is_file() && entry.path().extension().unwrap() == "capnp" {
      let metadata = fs::metadata(entry.path()).unwrap();
      let modified_time = FileTime::from_last_modification_time(&metadata).unix_seconds();

      if modified_time as u64 > last_build_time {
        schema_rebuild_required = true;
      }
    }
  }

  if schema_rebuild_required {
    println!("Schema rebuild required");
    build_capnp_func(input_schema_folder, output_schema_folder);
  } else {
    eprintln!("No schema rebuild required");
  }
}
