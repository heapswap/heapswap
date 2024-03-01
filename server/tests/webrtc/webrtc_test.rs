use std::io::Write;
use std::net::SocketAddr;
use std::str::FromStr;
use std::sync::Arc;

use anyhow::Result;
use tokio::sync::Mutex;
use tokio::time::Duration;
use webrtc::api::interceptor_registry::register_default_interceptors;
use webrtc::api::media_engine::MediaEngine;
use webrtc::api::APIBuilder;
use webrtc::data_channel::data_channel_message::DataChannelMessage;
use webrtc::ice_transport::ice_candidate::{RTCIceCandidate, RTCIceCandidateInit};
use webrtc::ice_transport::ice_server::RTCIceServer;
use webrtc::interceptor::registry::Registry;
use webrtc::peer_connection::configuration::RTCConfiguration;
use webrtc::peer_connection::peer_connection_state::RTCPeerConnectionState;
use webrtc::peer_connection::sdp::session_description::RTCSessionDescription;
use webrtc::peer_connection::{math_rand_alpha, RTCPeerConnection};
use axum::{
	response::{Html, Json},
	routing::{get, post, Router},
};

#[tokio::test]
async fn test_server(){
	let app = Router::new()
		.route("/webrtc/offer", get(offer_handler))
		.route("/webrtc/answer/:answer_id", get(answer_handler));
	
	let listener = tokio::net::TcpListener::bind("127.0.0.1:3000").await.unwrap();
	
	axum::serve(listener, app).await.unwrap();
}

#[tokio::test]
async fn test_client(){
	let offer = reqwest::get("http://localhost:3000/webrtc/offer").await.unwrap().text().await.unwrap();
	
	 	
		
		
	
}