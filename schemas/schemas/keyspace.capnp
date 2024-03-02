@0xe9d9a8a9e13421f5
using Rust = import "rust.capnp";
$Rust.parentModule("schemas");

# A keyspace is a 256 bit address space that holds vectors
struct KeySpace {
  # The space is a ed25519 public key
  keyspace_address @0 :List(Int64); 
  # The size (n) of the keyfield (n*64)
  keyvector_size @1 :Int32;
}
