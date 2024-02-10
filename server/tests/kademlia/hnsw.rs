use hnsw_rs::prelude::*;
use heapswap::macros::*;
use std::simd::{
	u32x1, u32x2, u32x4, u32x8, u32x16, u32x32,
    u64x1, u64x2, u64x4, u64x8, u64x16, u64x32,
};
use chrono::{DateTime, Utc};

type Hash32 = u32x1;
type Hash64 = u32x2;
type Hash128 = u32x4;
type Hash256 = u32x8;
type Hash512 = u32x16;
type Hash1024 = u32x32;


// struct Key { // 512-bit hash
// }

// struct UserAddress { // 256-bit hash
//     e: Hash256, // ed25519 public key
// }

// struct NodeAddress { 
// 	node_id: Hash128,
// 	ipv6: u32x4,
// }


// struct Entity { // 256-bit hash
//     a: Hash256, // ed25519 public key
// }

// struct World { // 128-bit hash
// 	w: Hash128,
// }

// struct Position { // 128-bit hash
//     a: u32,
//     b: u32,
//     c: u32,
//     d: u32,
// 	x: u32,
// 	y: u32,
// 	z: u32,
// 	t: u32,
// }

struct Key{
    o: u64x4,
    t: u64x4,
    w: u64x4,
    p: u64x4,
}

/*
    [Entity] (128 bits) - {Tags} (128 bits) = Entity Data
    [Component] (128 bits) - {Tags} (128 bits) = Component Data
    [System] (128 bits) - {Tags} (128 bits) = System Data

    DECA-V
    
    dimension.entity.component.attribute = value
    
    localhost.alice.velocity.x = 0
    
    localhost.alice.friends.bob = True
    localhost.bob.friends.alice = True
      

    localhost.* = [alice, bob, charlie]
    localhost.*.friends = [alice, bob]
    localhost.*.friends.bob = [alice]
    
    localhost.alice.* = [friends, velocity]
    localhost.alice.*.bob = [friends]
    localhost.alice.friends.* = [bob]
    
    *.alice = [localhost]
    *.alice.friends = [localhost]
    *.alice.friends.bob = [localhost]
    
    
    
    
    
    
    
    DataType
    Address   (Entity) - {Component} - <World> - [Data] 
    Tags       (Tags)       {Tags}      <Tags>   [Tags]
    
    
    E * * * -> C // The components that the entity has
    E C * * -> W // The worlds where the entity has the component
    E C W * -> D // The data for the entity-component-world
    
    E C W D -> E // Find similar entities
    
    
    
    
    
Entity
    get all worlds for agent
World
    get all agents in world


Entity + World = Position
    a w * -> * * p
    get the position for agent in world
    
World + Position = Entity
    * w p -> a * *
    get the agents in world at a specific position


    
World ? Position = Position
    find agents in world at a specific position
        

    
*/

struct InverseIndexItem {
    key: Key,
    updated_at: DateTime<Utc>,
}


struct Value {
    
}






#[test]
fn main() {
    // Create a new HNSW index
    let max_nb_connection = 16;
    let nb_elements = 1000;
    let nb_layers = 16;
    let ef_construction = 200;
    let mut hnsw = Hnsw::<u32, DistL2>::new(max_nb_connection, nb_elements, nb_layers, ef_construction, DistL2{});

    // Insert some data
    for i in 0..nb_elements {
        let data = vec![i as u32; 128]; // 128-dimensional vector
        hnsw.insert((&data, i));
    }

    // Perform a search
    let query = vec![0.5; 128]; // 128-dimensional query vector
    let ef_search = 64;
    let top_k = 10;
	
	
    let mut neighbours = hnsw.search(&query, top_k, ef_search);
	
	timeit!({
		neighbours = hnsw.search(&query, top_k, ef_search);
	});
	
    // Print the IDs and distances of the nearest neighbours
    for neighbour in neighbours {
        println!("ID: {}, Distance: {}", neighbour.d_id, neighbour.distance);
    }
}