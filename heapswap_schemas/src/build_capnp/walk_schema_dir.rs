use std::fs;
use std::path::Path;

//mod write_flattened_mod;
//use write_flattened_mod::write_flattened_mod;
use super::write_flattened_mod::write_flattened_mod;

// recursively walk through a directory and generate a list of all the schema files and folders
#[allow(dead_code)]
pub fn walk_schema_dir(input_dir_path_name: &str, output_dir_path_name: &str) -> Vec<String> {
  // this is the list of schema files and folders to reexport
  let mut reexport_list = Vec::new();

  // loop through the entries in the directory
  if let Ok(entries) = fs::read_dir(input_dir_path_name) {
    for entry in entries {
      if let Ok(entry) = entry {
        // get the path of the entry
        let path = entry.path();

        // get the name of the entry
        let entry_name = path
          .file_name()
          .and_then(|name| name.to_str())
          .map(|s| s.to_string())
          .unwrap_or_else(|| String::from("")); // might need to make this more robust

        // if the entry is a .capnp file
        if entry_name.ends_with(".capnp") {
          // add the file name to the reexport list if it's a .capnp file
          reexport_list.push(entry_name.replace(".", "_"));

        // if the entry is a directory
        } else if path.is_dir() {
          
          // check if the directory contains a .capnp file
          let mut entry_iter = std::fs::read_dir(&path).unwrap();
          if entry_iter.any(|entry| {
              let entry = entry.unwrap();
              let file_name = entry.file_name();
              let file_name_str = file_name.to_str().unwrap();
              file_name_str.ends_with(".capnp")
          }) {
            // the directory contains a .capnp file
            // add the directory name to the reexport list
            reexport_list.push(entry_name);
          }

          // switch the input and output directory paths
          let input_dir_path = Path::new(input_dir_path_name);
          let output_dir_path = Path::new(output_dir_path_name);
          
          if let Ok(stripped_path) = path.strip_prefix(input_dir_path) {
            // create a new path by joining the output directory path with the stripped path
            let new_path = output_dir_path.join(stripped_path);
            walk_schema_dir(
              path.display().to_string().as_str(),
              new_path.display().to_string().as_str(),
            );
          } else {
            eprintln!("Error stripping prefix from path");
          }
        }
      }
    }
  } else {
    eprintln!("Error reading directory: {}", input_dir_path_name);
  }

  // write the mod file (ignoring the io::Result)
  let _ = write_flattened_mod(format!("{output_dir_path_name}/mod.rs"), &reexport_list);

  return reexport_list;
}
