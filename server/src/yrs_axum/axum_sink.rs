use axum::extract::ws::{Message, WebSocket};
use futures_util::stream::SplitSink;
use std::pin::Pin;
use std::task::{Context, Poll};

// An axum websocket sink wrapper, that implements futures Sink in a way
// that makes it compatible with y-sync protocol,
// so that it can be used by y-sync crate BroadcastGroup.
#[repr(transparent)]
#[derive(Debug)]
pub struct AxumSink(SplitSink<WebSocket, Message>);

// enable conversion from SplitSink to AxumSink
impl From<SplitSink<WebSocket, Message>> for AxumSink {
	fn from(sink: SplitSink<WebSocket, Message>) -> Self {
		AxumSink(sink)
	}
}

// enable conversion from AxumStream to SplitStream
impl Into<SplitSink<WebSocket, Message>> for AxumSink {
	fn into(self) -> SplitSink<WebSocket, Message> {
		self.0 // this represents the websocket object
	}
}

// implement the actual wrapper
// the poll_ready, start_send, and poll_flush methods require a pinned receiver
// because they may be paused and resumed by the executor
impl futures_util::Sink<Vec<u8>> for AxumSink {
	type Error = y_sync::sync::Error;

	// see if the websocket sender is ready
	fn poll_ready(
		mut self: Pin<&mut Self>,
		cx: &mut Context<'_>,
	) -> Poll<Result<(), Self::Error>> {
		// use a pinned receiver
		match Pin::new(&mut self.0).poll_ready(cx) {
			// if the websocket sender is not ready, return Poll::Pending
			Poll::Pending => Poll::Pending,

			// if the websocket sender returns an error, return a y-sync error
			Poll::Ready(Err(e)) => {
				Poll::Ready(Err(y_sync::sync::Error::Other(e.into())))
			}

			// if the websocket sender is ready, return Poll::Ready(Ok(()))
			Poll::Ready(_) => Poll::Ready(Ok(())),
		}
	}

	// send a binary message to the client
	fn start_send(
		mut self: Pin<&mut Self>,
		item: Vec<u8>,
	) -> Result<(), Self::Error> {
		// use a pinned receiver
		if let Err(e) = Pin::new(&mut self.0)
			// send the message
			.start_send(axum::extract::ws::Message::Binary(item))
		{
			// if the message could not be sent, return an error
			Err(y_sync::sync::Error::Other(e.into()))
		} else {
			Ok(())
		}
	}

	// flush the websocket sender
	fn poll_flush(
		mut self: Pin<&mut Self>,
		cx: &mut Context<'_>,
	) -> Poll<Result<(), Self::Error>> {
		// use a pinned receiver
		match Pin::new(&mut self.0).poll_flush(cx) {
			// if the websocket sender is not ready, return Poll::Pending
			Poll::Pending => Poll::Pending,
			// if the websocket sender returns an error, return an y-sync error
			Poll::Ready(Err(e)) => {
				Poll::Ready(Err(y_sync::sync::Error::Other(e.into())))
			}
			// if the websocket sender is ready, return Ready(Ok(()))
			Poll::Ready(_) => Poll::Ready(Ok(())),
		}
	}

	// close the websocket sender
	fn poll_close(
		mut self: Pin<&mut Self>,
		cx: &mut Context<'_>,
	) -> Poll<Result<(), Self::Error>> {
		// use a pinned receiver
		match Pin::new(&mut self.0).poll_close(cx) {
			// if the websocket sender is not ready, return Poll::Pending
			Poll::Pending => Poll::Pending,
			// if the websocket sender returns an error, return an y-sync error
			Poll::Ready(Err(e)) => {
				Poll::Ready(Err(y_sync::sync::Error::Other(e.into())))
			}
			// if the websocket sender is ready, return Ready(Ok(()))
			Poll::Ready(_) => Poll::Ready(Ok(())),
		}
	}
}
