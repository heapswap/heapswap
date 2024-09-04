use crate::*;

/*
#[tokio::test]
async fn test_channel_manager_oneshot() -> Result<(), ChannelError> {
	let manager = ChannelManager::<String>::new();

	let first_message = "Hello".to_string();
	let second_message = "World".to_string();

	// oneshot - auto handle
	let handle = manager.create_oneshot();
	manager.send_oneshot(&handle, first_message.clone())?;
	let val = manager.recv_next_oneshot(&handle).await?;
	assert_eq!(val, first_message);

	// oneshot - custom handle
	let handle = manager.handle();
	manager.create_oneshot_with_handle(&handle);
	manager.send_oneshot(&handle, first_message.clone())?;
	let val = manager.recv_next_oneshot(&handle).await?;
	assert_eq!(val, first_message);

	Ok(())
}

#[tokio::test]
async fn test_unbounded_reexport() -> Result<(), ChannelError> {
	let (mut tx, mut rx) = unbounded::<String>();
	let _ = tx.send("Hello".to_string());
	let val = rx.next().await.unwrap();
	assert_eq!(val, "Hello".to_string());

	Ok(())
}

#[tokio::test]
async fn test_channel_manager_stream() -> Result<(), ChannelError> {
	let manager = ChannelManager::<String>::new();
	let first_message = "Hello".to_string();
	let second_message = "World".to_string();

	// stream - auto handle
	let handle = manager.create_stream();

	manager.send_stream(&handle, first_message.clone())?;
	manager.send_stream(&handle, second_message.clone())?;
	let val = manager.recv_next_stream(&handle).await?;
	assert_eq!(val, first_message);
	let val = manager.recv_next_stream(&handle).await?;
	assert_eq!(val, second_message);

	// stream - custom handle
	let handle = manager.handle();
	manager.create_stream_with_handle(&handle);

	manager.send_stream(&handle, first_message.clone())?;
	manager.send_stream(&handle, second_message.clone())?;
	let val = manager.recv_next_stream(&handle).await?;
	assert_eq!(val, first_message);
	let val = manager.recv_next_stream(&handle).await?;
	assert_eq!(val, second_message);

	Ok(())
}

#[tokio::test]
async fn test_channel_manager_oneshot_or_stream() -> Result<(), ChannelError> {
	let manager = ChannelManager::<String>::new();

	let first_message = "Hello".to_string();
	let second_message = "World".to_string();

	// auto handles
	let oneshot_handle = manager.create_oneshot();
	let stream_handle = manager.create_stream();

	manager.send_oneshot(&oneshot_handle, first_message.clone())?;
	manager.send_stream(&stream_handle, second_message.clone())?;

	let val = manager.recv_next_stream_or_oneshot(&oneshot_handle).await?;
	assert_eq!(val, first_message);

	let val = manager.recv_next_stream_or_oneshot(&stream_handle).await?;
	assert_eq!(val, second_message);

	// custom handles
	let oneshot_handle = manager.handle();
	manager.create_oneshot_with_handle(&oneshot_handle);
	let stream_handle = manager.handle();
	manager.create_stream_with_handle(&stream_handle);

	manager.send_oneshot(&oneshot_handle, first_message.clone())?;
	manager.send_stream(&stream_handle, second_message.clone())?;

	let val = manager.recv_next_stream_or_oneshot(&oneshot_handle).await?;
	assert_eq!(val, first_message);

	let val = manager.recv_next_stream_or_oneshot(&stream_handle).await?;
	assert_eq!(val, second_message);

	Ok(())
}
*/
