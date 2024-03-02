@0xdc2bf0501e93b775
using Rust = import "rust.capnp";
$Rust.parentModule("schemas");

struct KadBuckets {
  kad_buckets @0 :List(KadBucket);
}

struct KadBucket {
  nodes @0 :List(Node);
}