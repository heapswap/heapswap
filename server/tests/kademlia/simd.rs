use std::simd::{u64x4, ToBytes};
use rand::Rng;
use heapswap::macros::*;

const VECTOR_COUNT: usize = 256;
const VECTOR_SIZE: usize = 4;
type VectorSize = u64x4;

// hamming distance
fn hamming_distance(a: VectorSize, b: VectorSize) -> u32 {
    let mut total_ones = 0;
    let vector = a ^ b;
    
    for &value in vector.to_array().iter() {
        total_ones += value.count_ones();
    }
    
    total_ones
}

//euclidean distance
fn euclidean_distance(a: VectorSize, b: VectorSize) -> u32 {
    let mut total_ones = 0;
    let vector = a & b;
    
    for &value in vector.to_array().iter() {
        total_ones += value.count_ones();
    }
    
    total_ones
}


//#[test]
fn ints() {
    
    //wait for 5s
    std::thread::sleep(std::time::Duration::from_secs(5));
    
    let mut rng = rand::thread_rng();
    
    // generate random vectors
    let mut vectors: Vec<VectorSize> = Vec::with_capacity(VECTOR_COUNT);
    for _ in 0..VECTOR_COUNT {
        let mut vector = VectorSize::splat(0);
        for i in 0..VECTOR_SIZE {
            vector[i] = rng.gen(); 
        }
        vectors.push(vector);
    }
    
    // create a query vector
    let query = VectorSize::splat(0); 

    let mut closest = VectorSize::splat(u64::MAX);
    let mut closest_dist = u32::MAX;
    
    
    println!("simd hamming");
    timeit!({
    // find the closest vector
    closest = VectorSize::splat(u64::MAX);
    closest_dist = u32::MAX;
    for vector in vectors.clone() {
        let dist = hamming_distance(query, vector);
        if dist < closest_dist {
            closest_dist = dist;
            closest = vector;
        }
    }
    });
    
    println!("Closest vector: {:?}", closest);
    println!("Distance: {}", closest_dist);
    
    
    println!("simd euclidean");
    timeit!({
    // find the closest vector
    closest = VectorSize::splat(u64::MAX);
    closest_dist = u32::MAX;
    for vector in vectors.clone() {
        let dist = euclidean_distance(query, vector);
        if dist < closest_dist {
            closest_dist = dist;
            closest = vector;
        }
    }
    });
    
    println!("Closest vector: {:?}", closest);
    println!("Distance: {}", closest_dist);
    
}


// hamming distance
fn hamming_distance_for_loop(a: VectorSize, b: VectorSize) -> u32 {
    let mut total_ones = 0;
    let vector = a ^ b;
    
    for &value in vector.to_array().iter() {
        total_ones += value.count_ones();
    }
    
    total_ones
}
 
// hamming distance
fn hamming_distance_sequential(a: VectorSize, b: VectorSize) -> u32 {
    (a ^ b)
        .to_array()
        .iter()
        .map(|&value| value.count_ones())
        .sum()
}

#[test]
fn for_loop_vs_sequential(){
    
    let mut rng = rand::thread_rng();
    
    // generate random vectors
    let mut vectors: Vec<VectorSize> = Vec::with_capacity(VECTOR_COUNT);
    for _ in 0..VECTOR_COUNT {
        let mut vector = VectorSize::splat(0);
        for i in 0..VECTOR_SIZE {
            vector[i] = rng.gen(); 
        }
        vectors.push(vector);
    }
    
    // create a query vector
    let query = VectorSize::splat(0); 

    let mut closest = VectorSize::splat(u64::MAX);
    let mut closest_dist = u32::MAX;
    
    println!("simd for loop");
    timeit!({
    // find the closest vector
    closest = VectorSize::splat(u64::MAX);
    closest_dist = u32::MAX;
    for vector in vectors.clone() {
        let dist = hamming_distance_for_loop(query, vector);
        if dist < closest_dist {
            closest_dist = dist;
            closest = vector;
        }
    }
    });
    
    println!("Closest vector: {:?}", closest);
    println!("Distance: {}", closest_dist);
    
    
    println!("simd sequential");
    timeit!({
    // find the closest vector
    closest = VectorSize::splat(u64::MAX);
    closest_dist = u32::MAX;
    for vector in vectors.clone() {
        let dist = hamming_distance_sequential(query, vector);
        if dist < closest_dist {
            closest_dist = dist;
            closest = vector;
        }
    }
    });
    
    println!("Closest vector: {:?}", closest);
    println!("Distance: {}", closest_dist);
    
}