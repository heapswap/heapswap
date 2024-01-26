use memory_stats::memory_stats;
use human_bytes::human_bytes;
use std::alloc::Layout;

#[test]
fn main() {
	let layout = Layout::new::<u64>();
    println!("Size: {}", layout.size());
    println!("Alignment: {}", layout.align());	
	
    if let Some(usage) = memory_stats() {
        println!("Current physical memory usage: {}", human_bytes(usage.physical_mem as f64));
        println!("Current virtual memory usage: {}", human_bytes(usage.virtual_mem as f64));
    } else {
        println!("Couldn't get the current memory usage :(");
    }
}