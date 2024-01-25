use heapswap::serve::serve;

#[tokio::test]
async fn test_server(){
	serve(8000).await;
}