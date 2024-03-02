@0xebae124d97d2183d;

using Rust = import "rust.capnp";
$Rust.parentModule("schemas");


struct Node {
  node_id @0 :NodeID;
  node_address @1 :NodeAddress;
  node_keyspaces @2 :List(Keyspace);
  node_ping_ms @3 :Int32;
} 

struct NodeID{   
  # The NodeId is an ed25519 public key
  node_id @0 :List(Int64);
}

struct NodeAddress {
  # The (optional) ipv4 address 
  ipv4_address @1 :List(Int8);
  ipv4_port @2 :Int16;
  
  # The (optional) ipv6 address
  ipv6_address @3 :List(Int8);
  ipv6_port @4 :Int16;
}


