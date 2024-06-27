pub mod fruits { 
	include!(concat!(env!("OUT_DIR"), "/fruits.rs")); 
}

pub mod snazzy {
	pub mod shirt { 
		include!(concat!(env!("OUT_DIR"), "/snazzy.shirt.rs")); 
		
		pub mod button { 
			include!(concat!(env!("OUT_DIR"), "/snazzy.shirt.button.rs")); 
		}
	}
	
	pub mod pants { 
		include!(concat!(env!("OUT_DIR"), "/snazzy.pants.rs")); 
	}
}

pub mod shoes { 
	pub mod shoes { 
		pub mod shoes { 
			pub mod shoes { 
				include!(concat!(env!("OUT_DIR"), "/shoes.shoes.shoes.shoes.rs")); 
			}
		}
	}
}