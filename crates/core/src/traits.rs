use bytes::Bytes;

pub trait Byteable<E>: Sized {
    fn to_bytes(&self) -> Bytes;
    fn from_bytes(bytes: &Bytes) -> Result<Self, E>;
}

pub trait Stringable<E>: Sized {
    fn to_string(&self) -> String;
    fn from_string(string: &str) -> Result<Self, E>;
}

pub trait Randomable<E>: Sized {
    fn from_random() -> Result<Self, E>;
}
