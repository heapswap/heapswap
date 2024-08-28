use crate::*;

#[tokio::test]
async fn test_portal_manager_oneshot() -> Result<(), PortalError> {
	let manager = PortalManager::<String>::new();

	let first_message = "Hello".to_string();
	let second_message = "World".to_string();

	// oneshot - auto handle
	let handle = manager.create_oneshot();
	manager.send_oneshot(handle, first_message.clone())?;
	let val = manager.recv_oneshot(handle).await?;
	assert_eq!(val, first_message);

	// oneshot - custom handle
	let handle = manager.handle();
	manager.create_oneshot_with_handle(handle);
	manager.send_oneshot(handle, first_message.clone())?;
	let val = manager.recv_oneshot(handle).await?;
	assert_eq!(val, first_message);

	Ok(())
}

#[tokio::test]
async fn test_unbounded_reexport() -> Result<(), PortalError> {
	let (mut tx, mut rx) = unbounded::<String>();
	let _ = tx.send("Hello".to_string());
	let val = rx.next().await.unwrap();
	assert_eq!(val, "Hello".to_string());

	Ok(())
}

#[tokio::test]
async fn test_portal_manager_stream() -> Result<(), PortalError> {
	let manager = PortalManager::<String>::new();
	let first_message = "Hello".to_string();
	let second_message = "World".to_string();

	// stream - auto handle
	let handle = manager.create_stream();

	manager.send_stream(handle, first_message.clone())?;
	manager.send_stream(handle, second_message.clone())?;
	let val = manager.recv_stream(handle).await?;
	assert_eq!(val, first_message);
	let val = manager.recv_stream(handle).await?;
	assert_eq!(val, second_message);

	// stream - custom handle
	let handle = manager.handle();
	manager.create_stream_with_handle(handle);

	manager.send_stream(handle, first_message.clone())?;
	manager.send_stream(handle, second_message.clone())?;
	let val = manager.recv_stream(handle).await?;
	assert_eq!(val, first_message);
	let val = manager.recv_stream(handle).await?;
	assert_eq!(val, second_message);

	Ok(())
}
