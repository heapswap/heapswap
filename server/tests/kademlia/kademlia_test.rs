use rand::Rng;
use std::cmp::Ordering;
use std::rc::Rc;
use heapswap::{
	macros::*,
};

const KEY_LENGTH: usize = 4;

#[derive(Clone)]
struct Node {
    parent: Option<Rc<Node>>,
    address: [u64; KEY_LENGTH],
	ping: f64,
    data: String,
}

impl Ord for Node {
    fn cmp(&self, other: &Self) -> Ordering {
        match &self.parent {
            Some(parent) => xor_distance_metric(self.address, other.address).cmp(&xor_distance_metric(self.address, parent.address)),
            None => Ordering::Equal,
        }
    }
}

impl PartialOrd for Node {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Eq for Node {}

impl PartialEq for Node {
    fn eq(&self, other: &Self) -> bool {
        self.address == other.address
    }
}
/*
// works
fn distance_metric(a: [u64; KEY_LENGTH], b: [u64; KEY_LENGTH]) -> u64 {
    let mut distance = 0; 
    for i in 0..KEY_LENGTH {
        distance += (a[i] ^ b[i]).count_ones() as u64;
    }
    distance
}
*/       
fn xor_distance_metric(a: [u64; KEY_LENGTH], b: [u64; KEY_LENGTH]) -> u64 {
    let mut distance = 0; 
    for i in 0..KEY_LENGTH {
        distance += (a[i] ^ b[i]).count_ones() as u64;
    }
    distance
}

// seems to just be a worse version of xor_distance_metric 
fn and_distance_metric(a: [u64; KEY_LENGTH], b: [u64; KEY_LENGTH]) -> u64 {
    let mut distance = 0; 
    for i in 0..KEY_LENGTH {
        distance += (a[i] & b[i]).count_ones() as u64;
    }
    distance
}

fn generate_address() -> [u64; KEY_LENGTH] {
    let mut rng = rand::thread_rng();
    let mut address = [0; KEY_LENGTH];
    for i in 0..KEY_LENGTH {
        address[i] = rng.gen::<u64>();
    }
    address
}

fn address_to_binary_string(address: [u64; KEY_LENGTH]) -> String {
    address.iter().map(|x| format!("{:064b}", x)).collect::<Vec<String>>().join("")
}

fn flip_random_bits(address: &mut [u64; KEY_LENGTH], n: usize) {
    let mut rng = rand::thread_rng();
    for _ in 0..n {
        let index = rng.gen_range(0..KEY_LENGTH);
        let bit = 1 << rng.gen_range(0..64);
        address[index] ^= bit;
    }
}


const MAX_PING: f64 = 500.0;
const MAX_DIST: f64 = 256.0;

#[test]
fn main() {
    let self_node = Node {
        parent: None,
		ping: 0.0,
        address: generate_address(),
        data: "self".to_string(),
    };
    
    let mut other_nodes = Vec::new();
	
	let mut rng = rand::thread_rng();
    
    //other_nodes.push(self_node.clone());
    
    
    for i in 0..255 {
        other_nodes.push(Node {
            parent: Some(Rc::new(self_node.clone())),
			ping: rng.gen::<f64>() * MAX_PING,
            address: generate_address(),
            data: format!("child-{}", i),
        });
    }
    
    
    // works
    other_nodes.sort_by(|a, b| xor_distance_metric(self_node.address, a.address).cmp(&xor_distance_metric(self_node.address, b.address)));
            
    
    fn ping_metric(node_a: &Node, node_b: &Node) -> f64 {
        // (MaxPing - Ping) * (MaxDist - Distance)
        
        let distance = xor_distance_metric(node_a.address, node_b.address);
        
        return 0.0 + ((MAX_PING - node_b.ping) * (MAX_DIST - distance as f64));
    }     
    
    /*
    other_nodes.sort_by(|a, b| ping_metric(&self_node, &a).partial_cmp(&ping_metric(&self_node, &b)).unwrap_or(Ordering::Equal));
    */
    
    
        
	/*
    // print self
    println!("Self:\t{}", address_to_binary_string(self_node.address));   
    */
	
	/*
    // print distances 
    for node in &mut other_nodes {
        println!("{:?}\t{}", distance_metric(self_node.address, node.address), address_to_binary_string(node.address));
    }
	*/
    
	  
    // brute force find absolute closest to query address
	
    // completely random query address
    let query_address = generate_address();
    
    // query address similar to self node address
    //let mut query_address = self_node.address.clone(); 
    //flip_random_bits(&mut query_address, 256); // Flip 5 random bits   
    
    println!("Query distance from self: {:?}", xor_distance_metric(self_node.address, query_address)); 
        
    let mut query_nodes = other_nodes.clone();
	
    
    query_nodes.sort_by(|a, b| xor_distance_metric(query_address, a.address).cmp(&xor_distance_metric(query_address, b.address)));
	
    /*
    query_nodes.sort_by(|a, b| ping_metric(&self_node, &a).partial_cmp(&ping_metric(&self_node, &b)).unwrap_or(Ordering::Equal));
    */	
    println!("Forced   (distance {:?}) (ping {:.2}) is {}",  xor_distance_metric(query_address, query_nodes[0].address),
    query_nodes[0].ping, address_to_binary_string(query_nodes[0].address));
	
	
	let mut closest_node :Option<&Node> = None;
	let closest_node = binary_search_closest(&other_nodes, query_address);
	match closest_node {
        Some(node) => println!("Searched (distance {:?}) (ping {:.2}) is {}",  xor_distance_metric(query_address, node.address), node.ping, address_to_binary_string(node.address)),
        None => println!("No nodes found"),
    }
	
    // find the rank of the searched node
    let mut rank = 0;
    for node in &query_nodes {
        if node.address == closest_node.unwrap().address {
            break;
        }
        rank += 1;
    }
    println!("Rank: {}", rank);   
    
}
 
fn binary_search_closest(nodes: &[Node], query_address: [u64; KEY_LENGTH]) -> Option<&Node> {
    if nodes.is_empty() {
        return None;
    } 

	if nodes.len() == 1 {
        return Some(&nodes[0]);
    }   
    
    let mid = nodes.len() / 2;    
    let mid_node = &nodes[mid];
    
    let (left_half, right_half) = nodes.split_at(mid);
    
    // let left_mid = left_half.len() / 2;
    // let left_mid_node = &left_half[left_mid];
	let left_node = &left_half[0];
    let left_distance = xor_distance_metric(query_address, left_node.address);
    
    // let right_mid = right_half.len() / 2;
    // let right_mid_node = &right_half[right_mid];
	let right_node = &right_half[right_half.len()-1];
    let right_distance = xor_distance_metric(query_address, right_node.address);
    
    if left_distance <= right_distance {
        return binary_search_closest(left_half, query_address);
    } else {
        return binary_search_closest(right_half, query_address);
    } 
  	    
}


/*
// sort of works
fn binary_search_closest(nodes: &[Node], query_address: [u64; KEY_LENGTH]) -> Option<&Node> {
    if nodes.is_empty() {
        return None;
    } 

	if nodes.len() == 1 {
        return Some(&nodes[0]);
    }   
    
    let mid = nodes.len() / 2;    
    let mid_node = &nodes[mid];
  
    let mid_distance = distance_metric(query_address, mid_node.address);

    let (left_half, right_half) = nodes.split_at(mid);
    
	let left_node = &left_half[0];
	let right_node = &right_half[right_half.len()-1];
    	
	let left_distance = distance_metric(query_address, left_node.address);
    let right_distance = distance_metric(query_address, right_node.address);  
	

    if left_distance <= mid_distance {
        return binary_search_closest(left_half, query_address);
    } else {
        return binary_search_closest(right_half, query_address);
    } 
  	
}
*/