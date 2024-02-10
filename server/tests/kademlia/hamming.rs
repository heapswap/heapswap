use rand::Rng;
use heapswap::macros::*;

#[derive(Clone, Copy)]
struct Address{
	a: u64,
	b: u64,
	c: u64,
	d: u64,
}

impl Address{
	fn new() -> Self{
		let mut rng = rand::thread_rng();
		Self{
			a: rng.gen(),
			b: rng.gen(),
			c: rng.gen(),
			d: rng.gen(),
		}
	}
}

fn xor(a: u64, b: u64) -> u32 {
	(a ^ b).count_ones()
}

fn xor_distance(a: Address, b: Address) -> u32 {
	xor(a.a, b.a) + xor(a.b, b.b) + xor(a.c, b.c) + xor(a.d, b.d)
}

fn euclidean(a: u64, b: u64) -> u32 {
	(a & b).count_ones()
}

fn euclidean_distance(a: Address, b: Address) -> u32 {
	euclidean(a.a, b.a) + euclidean(a.b, b.b) + euclidean(a.c, b.c) + euclidean(a.d, b.d)
}


#[test]
fn main(){
	
	let self_address = Address::new();
	
	const ADDRESS_COUNT: usize = 256;
	// create array of addresses
	let mut addresses = Vec::with_capacity(ADDRESS_COUNT);
	for _ in 0..ADDRESS_COUNT {
		addresses.push(Address::new());
	}
	
	println!("Hamming");
	
	// sort by distance to self_address
	timeit!({
	addresses.clone().sort_by(|a, b| xor_distance(self_address, *a).partial_cmp(&xor_distance(self_address, *b)).unwrap());
	});
	
	// find the closest address
	let mut min_dist = u32::MAX;
	timeit !({
		min_dist = u32::MAX;
		for address in addresses.clone() {
			let dist = xor_distance(self_address, address);
			if dist < min_dist {
				min_dist = dist;
			}
		}
	});
	println!("Min Distance: {}", min_dist);
	
	
	println!("Euclidean");
	
	// sort by distance to self_address
	timeit!({
	addresses.clone().sort_by(|a, b| euclidean_distance(self_address, *a).partial_cmp(&euclidean_distance(self_address, *b)).unwrap());
	});
	
	// find the closest address
	min_dist = u32::MAX;
	timeit !({
		min_dist = u32::MAX;
		for address in addresses.clone() {
			let dist = euclidean_distance(self_address, address);
			if dist < min_dist {
				min_dist = dist;
			}
		}
	});
	
	println!("Min Distance: {}", min_dist);
	
}