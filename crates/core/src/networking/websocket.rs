
use std::{net::{Ipv4Addr, Ipv6Addr}, sync::Arc};

use futures::{stream::{SplitSink, SplitStream}, AsyncRead, AsyncWrite, SinkExt, StreamExt};
use gloo::{
	net::websocket::{events::CloseEvent, futures::WebSocket, Message},
	timers::callback::Timeout,
};
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::spawn_local;
use wasm_bindgen::JsCast;
use web_sys::window;
use web_sys::Event;
use crate::crypto::{noise::*, keys::*};
use std::pin::Pin;
use std::task::{Context, Poll};
use bytes::{Buf, BufMut, BytesMut};
use tokio::sync::{Mutex, MutexGuard};

#[derive(Debug)]
pub enum WebSocketError {
	InvalidWebsocketObject,
	WebSocketError,
}

pub struct WebSocketWrapper {
    write: Arc<Mutex<SplitSink<WebSocket, Message>>>,
    read: Arc<Mutex<SplitStream<WebSocket>>>,
}

fn websys_websocket_to_gloo_websocket(ws: web_sys::WebSocket) -> Result<WebSocket, WebSocketError> {
	let ws: web_sys::WebSocket = ws.dyn_into::<web_sys::WebSocket>().map_err(|_| WebSocketError::InvalidWebsocketObject)?;
	let ws: WebSocket = WebSocket::try_from(ws).map_err(|_| WebSocketError::InvalidWebsocketObject)?;
	Ok(ws)
}

impl WebSocketWrapper {
    pub fn new(ws: web_sys::WebSocket) -> Self {
		let ws = websys_websocket_to_gloo_websocket(ws).unwrap();
        let (write, read) = ws.split();
        Self { write: Arc::new(Mutex::new(write)), read: Arc::new(Mutex::new(read)) }
    }
    
    async fn write(&self) -> Arc<Mutex<SplitSink<WebSocket, Message>>> {
        self.write.clone()
    }
    
    async fn read(&self) -> Arc<Mutex<SplitStream<WebSocket>>> {
        self.read.clone()
    }
}

impl AsyncRead for WebSocketWrapper {
    fn poll_read(self: Pin<&mut Self>, cx: &mut Context<'_>, buf: &mut [u8]) -> Poll<std::io::Result<usize>> {
        let mut read = match self.read.try_lock() {
            Ok(read) => read,
            Err(_) => {
                cx.waker().wake_by_ref();
                return Poll::Pending;
            }
        };

        match read.poll_next_unpin(cx) {
            Poll::Ready(Some(Ok(Message::Text(text)))) => {
                let bytes = text.as_bytes();
                let len = bytes.len().min(buf.len());
                buf[..len].copy_from_slice(&bytes[..len]);
                Poll::Ready(Ok(len))
            }
            Poll::Ready(Some(Ok(Message::Bytes(bytes)))) => {
                let len = bytes.len().min(buf.len());
                buf[..len].copy_from_slice(&bytes[..len]);
                Poll::Ready(Ok(len))
            }
            Poll::Ready(Some(Err(_))) => Poll::Ready(Err(std::io::Error::new(std::io::ErrorKind::Other, "WebSocket error"))),
            Poll::Ready(None) => Poll::Ready(Err(std::io::Error::new(std::io::ErrorKind::UnexpectedEof, "WebSocket closed"))),
            Poll::Pending => Poll::Pending,
        }
    }
}

impl AsyncWrite for WebSocketWrapper {
    fn poll_write(self: Pin<&mut Self>, cx: &mut Context<'_>, buf: &[u8]) -> Poll<std::io::Result<usize>> {
        let mut write = match self.write.try_lock() {
            Ok(write) => write,
            Err(_) => {
                cx.waker().wake_by_ref();
                return Poll::Pending;
            }
        };

        let message = Message::Bytes(buf.to_vec());
        match write.poll_ready_unpin(cx) {
            Poll::Ready(Ok(())) => match write.start_send_unpin(message) {
                Ok(_) => Poll::Ready(Ok(buf.len())),
                Err(_) => Poll::Ready(Err(std::io::Error::new(std::io::ErrorKind::Other, "WebSocket write error"))),
            },
            Poll::Ready(Err(_)) => Poll::Ready(Err(std::io::Error::new(std::io::ErrorKind::Other, "WebSocket not ready"))),
            Poll::Pending => Poll::Pending,
        }
    }

    fn poll_flush(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<std::io::Result<()>> {
        let mut write = match self.write.try_lock() {
            Ok(write) => write,
            Err(_) => {
                cx.waker().wake_by_ref();
                return Poll::Pending;
            }
        };

        match write.poll_flush_unpin(cx) {
            Poll::Ready(Ok(())) => Poll::Ready(Ok(())),
            Poll::Ready(Err(_)) => Poll::Ready(Err(std::io::Error::new(std::io::ErrorKind::Other, "WebSocket flush error"))),
            Poll::Pending => Poll::Pending,
        }
    }

    fn poll_close(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<std::io::Result<()>> {
        let mut write = match self.write.try_lock() {
            Ok(write) => write,
            Err(_) => {
                cx.waker().wake_by_ref();
                return Poll::Pending;
            }
        };

        match write.poll_close_unpin(cx) {
            Poll::Ready(Ok(())) => Poll::Ready(Ok(())),
            Poll::Ready(Err(_)) => Poll::Ready(Err(std::io::Error::new(std::io::ErrorKind::Other, "WebSocket close error"))),
            Poll::Pending => Poll::Pending,
        }
    }
}