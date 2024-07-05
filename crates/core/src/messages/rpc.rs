use super::messages::*;

/* The four types of Services are:
	1. Unary RPCs: A single request followed by a single response.
	2. Server streaming RPCs: A single request followed by a sequence of responses.
	3. Client streaming RPCs: A sequence of requests followed by a single response.
	4. Bidirectional streaming RPCs: A sequence of requests and responses where the responses are interleaved with the requests.
*/

pub enum Input<I> {
	Unary(I),
	Streaming(I),
}

pub enum Output<O> {
	Unary(O),
	Streaming(O),
}

pub enum ServiceType {
	Unary,
	ServerStreaming,
	ClientStreaming,
	BidirectionalStreaming,
}

pub trait Service<I, O> {
	fn new(action: Action);
	fn action() -> Action;
	fn input(input: I) -> Output<O>;
	fn output(output: O);
}
