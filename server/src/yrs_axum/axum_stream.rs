use axum::extract::ws::WebSocket;
use futures_util::stream::SplitStream;
use std::pin::Pin;
use std::task::{Context, Poll};
use tokio_stream::Stream;

// A warp websocket stream wrapper, that implements futures Stream in a way
// that makes it compatible with y-sync protocol,
// so that it can be used by y-sync crate BroadcastGroup
#[derive(Debug)]
pub struct AxumStream(SplitStream<WebSocket>);

// enable conversion from SplitStream to AxumStream
impl From<SplitStream<WebSocket>> for AxumStream {
	fn from(stream: SplitStream<WebSocket>) -> Self {
		AxumStream(stream)
	}
}

// enable conversion from AxumStream to SplitStream
impl Into<SplitStream<WebSocket>> for AxumStream {
	fn into(self) -> SplitStream<WebSocket> {
		self.0 // this represents the websocket object
	}
}

// implement the actual wrapper
// the poll_next method requires a pinned receiver
// because it may be paused and resumed by the executor
impl Stream for AxumStream {
	type Item = Result<Vec<u8>, axum::Error>;

	// send a binary message to the client
	fn poll_next(
		mut self: Pin<&mut Self>,
		cx: &mut Context<'_>,
	) -> Poll<Option<Self::Item>> {
		// use a pinned receiver
		match Pin::new(&mut self.0).poll_next(cx) {
			// if the poll is pending, wait
			Poll::Pending => Poll::Pending,

			// if the poll is ready and has no message, return Ready(None)
			Poll::Ready(None) => Poll::Ready(None),

			// if the poll is ready and has a message,
			Poll::Ready(Some(res)) => match res {
				Ok(axum::extract::ws::Message::Binary(bin)) => {
					Poll::Ready(Some(Ok(bin)))
				}

				// If the message is not binary, return an error
				Ok(_) => Poll::Ready(Some(Err(axum::Error::new(
					std::io::Error::new(
						std::io::ErrorKind::Other,
						"AxumStream: non-binary message received",
					),
				)))),

				// otherwise, return an error
				Err(e) => Poll::Ready(Some(Err(e))),
			},
		}
	}
}
