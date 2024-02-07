use instant_distance::{Builder, Search};
use rand::Rng;
use heapswap::macros::*;

const POINT_LENGTH: usize = 4;

#[derive(Clone, Copy, Debug)]
struct Point([u64; POINT_LENGTH]);

impl instant_distance::Point for Point {
    fn distance(&self, other: &Self) -> f32 {
        // Hamming distance metric
        let mut distance = 0;
        for i in 0..POINT_LENGTH {
            distance += (self.0[i] ^ other.0[i]).count_ones() as u64;
        }
        distance as f32
    }
}

#[test]
fn main() {
    let mut rng = rand::thread_rng();
    let mut points = vec![];
    let mut values = vec![]; // Define values

    for _ in 0..256 {
        let point = Point([rng.gen(), rng.gen(), rng.gen(), rng.gen()]);
        points.push(point);
        values.push(point); // Populate values
    }

    let map = Builder::default().build(points, values);
    let mut search = Search::default();

    let cambridge_blue = Point([0, 0, 0, 0]);

	timeit!({
		let closest_point = map.search(&cambridge_blue, &mut search).next().unwrap();
	});
	
	let closest_point = map.search(&cambridge_blue, &mut search).next().unwrap();
	
    println!("{:?}", closest_point.value);
}