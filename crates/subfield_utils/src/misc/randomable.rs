use crate::*;
use std::net::{Ipv4Addr, Ipv6Addr};

// Able to be randomly generated
pub trait Randomable: Sized {
	fn random() -> Self;
}

// Able to be randomly generated
pub trait RandomLengthable: Sized {
	fn random_length(length: usize) -> Self;
}

impl Randomable for Ipv4Addr {
	fn random() -> Self {
		Ipv4Addr::new(random(), random(), random(), random())
	}
}

impl Randomable for Ipv6Addr {
	fn random() -> Self {
		Ipv6Addr::new(
			random(),
			random(),
			random(),
			random(),
			random(),
			random(),
			random(),
			random(),
		)
	}
}
