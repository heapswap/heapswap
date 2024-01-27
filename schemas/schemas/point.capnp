@0xebae124d97d2183d;

using Rust = import "rust.capnp";
$Rust.parentModule("schemas");

struct Point {
    x @0 :Int32;
    y @1 :Int32;
} 