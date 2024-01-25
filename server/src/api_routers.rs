use crate::app_state::GlobalAppState;
use crate::yrs_axum::yjs_handlers::{
	get_yjs_default_room_handler, get_yjs_named_room_handler,
};
use axum::{routing::get, Router};

/*
api/v0
*/

// the api v0 router
pub fn api_v0_router() -> Router<GlobalAppState> {
	// the api routes
	Router::new()
		// default ping route
		// will get replaced with a health check later
		.route("/ping", get(|| async { "pong" }))
		// the yjs routes
		.nest("/yjs", yjs_v0_router())
		// the yrs routes
		.nest("/yrs", yrs_v0_router())
}

// connect between yjs and yrs (server-client)
pub fn yjs_v0_router() -> Router<GlobalAppState> {
	Router::new()
		// connect to the default room
		.route("/", get(get_yjs_default_room_handler))
		.route("/:room_name", get(get_yjs_named_room_handler))
}

// connect between yrs to yrs (server-server)
pub fn yrs_v0_router() -> Router<GlobalAppState> {
	Router::new()
}
