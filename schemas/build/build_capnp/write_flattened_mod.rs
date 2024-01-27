extern crate capnpc;

use std::fs;
use std::io;

// write a mod file that reexports all the schemas
pub fn write_flattened_mod(
  output_filepath: String, // the path to the mod file to write (including the filename)
  input_vec: &Vec<String>, // the list of all the schema files and folders to reexport
) -> io::Result<()> {
  // the string to write to the mod file
  let mut output_string = String::new();

  // loop through the list of schema files and folders and add them to the output string
  for entry in input_vec {
    // this mod statement could be pub if you wanted to be able to group schemas by folder
    output_string.push_str(&format!("pub mod {};\n", entry));
    output_string.push_str(&format!("pub use {}::*;\n", entry));
    output_string.push_str("\n");
  }

  // debugging
  // println!("output_filepath: {}", output_filepath);
  // println!("output_string: {}", output_string);

  // write the output string to the output file
  fs::write(output_filepath, output_string)?;

  // return an empty io::Result
  Ok(())
}
