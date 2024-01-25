use heapswap::serve::serve;

#[tokio::main]
pub async fn main() {
	serve(8000).await;
}
