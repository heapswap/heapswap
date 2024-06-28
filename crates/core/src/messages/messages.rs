use bytes::Bytes;

struct U256 {
    u0: u64,
    u1: u64,
    u2: u64,
    u3: u64,
}

type Key = U256;
type Hash = U256;

enum Action {
    // REST
    Post = 0,
    Get = 1,
    Delete = 2,
    Response = 3,

    // Pubsub
    Subscribe = 32,
    Unsubscribe = 33,
    Message = 34,
}

struct Path {
    signer: Option<Key>,
    cosigner: Option<Key>,
    tangent: Option<Hash>,
}

struct Request {
    id: U256,
    action: Action,
    path: Path,
    data: Bytes,
}

struct Response {
    id: U256,
    action: Action,
    path: Path,
    data: Bytes,
}
