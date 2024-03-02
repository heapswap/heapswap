@0xa93d6bc224e0c413
using Rust = import "rust.capnp";
$Rust.parentModule("schemas");

# A keyvector is an arbitrary length vector within a keyspace 
struct KeyVector {
  # Vectors are fetched:
  # - exactly
  # - nearest-n with the hamming distance
  keyvector_address @0 :List(Int64);
  keyvector_owner @3 :NodeID;
  keyvector_state @1 :KeyVectorState;
  keyvector_data @2 :KeyVectorData;
}

# KeyVectorState is the state of a keyvector
enum KeyVectorState {
  # The keyvector is available
  AVAILABLE @0;
  # The keyvector has been checked and does not exist
  NOT_FOUND @1;
  # The keyvector is being fetched
  FETCHING @2;
  # The keyvector is checked out
  CHECKED_OUT @3;
}

# KeyVectorData is the binary value of a keyvector
struct KeyVectorData; {
  metadata @0 :Data;
  data @1 :Data;
}