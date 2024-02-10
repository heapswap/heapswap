use {
    heapswap::macros::*, nalgebra::{distance, point, OPoint, Point, Point1, Point2, Point3, Point4, Point5, Point6, Scalar}, rand::Rng
};


fn hamming<T: Scalar, const D: usize>(x: &Point<T, D>, y: &Point<T, D>) -> u32
{
	x.iter().zip(y.iter()).fold(0, |acc, (a, b)| acc + (a != b) as u32)	
	
}

/*
100 loops: 1.9889364999999999 ms
nalgebra
1000 loops: 134.83404800000002 µs
Hamming
10000 loops: 73.579589 µs
10000 loops: 36.443116700000004 µs
*/


#[test]
fn main(){
    
    let mut rng = rand::thread_rng();
    
    // Build using components directly.
    let self_point = Point4::new(rng.gen::<f64>(), rng.gen::<f64>(), rng.gen::<f64>(), rng.gen::<f64>(),  );

	// create array of 10 points
	let point_count = 256;
	let mut points = Vec::with_capacity(point_count);
	for _ in 0..point_count {
		points.push(Point4::new(rng.gen::<f64>(), rng.gen::<f64>(), rng.gen::<f64>(), rng.gen::<f64>(),  ));
	}	
	
	// sort by distance to self_point
	timeit!({
	points.clone().sort_by(|a, b| distance(&self_point, a).partial_cmp(&distance(&self_point, b)).unwrap());
	});
	

    let mut min_dist = f64::MAX;
	println!("nalgebra");
    timeit!({
		// find the closest point
		min_dist = f64::MAX;
		for point in points.clone() {
			let dist = distance(&self_point, &point);
			if dist < min_dist {
				min_dist = dist;
			}
		}
    });
    //println!("Min Distance: {}", min_dist);
	
	
	println!("Hamming");
	// sort by distance to self_point
	timeit!({
	points.clone().sort_by(|a, b| hamming(&self_point, a).partial_cmp(&hamming(&self_point, b)).unwrap());
	});
	
	// find the closest point
	let mut min_dist = u32::MAX;
	timeit !({
		min_dist = u32::MAX;
		for point in points.clone() {
			let dist = hamming(&self_point, &point);
			if dist < min_dist {
				min_dist = dist;
			}
		}
	});
	
	
}

