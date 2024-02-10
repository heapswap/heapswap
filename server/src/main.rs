#![feature(portable_simd)]

// for dalek
// #![feature(stdarch_x86_avv512)]

use heapswap::serve::serve;

#[tokio::main]
pub async fn main() {
	serve(8000).await;
}
