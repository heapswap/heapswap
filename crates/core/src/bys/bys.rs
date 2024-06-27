use base32::Alphabet;
use bytes::Bytes;
use rand::Rng;

pub enum BytesError {
    InvalidBase32,
}

/**
 * Conversions
*/

pub fn to_string(data: &Bytes) -> String {
    String::from_utf8(data.to_vec()).unwrap()
}

pub fn from_string(data: &String) -> Bytes {
    Bytes::from(data.clone().into_bytes())
}

pub fn to_base32(data: &Bytes) -> String {
    base32::encode(Alphabet::Rfc4648Lower { padding: false }, &data)
}

pub fn from_base32(data: &str) -> Result<Bytes, BytesError> {
    base32::decode(Alphabet::Rfc4648Lower { padding: false }, data)
        .map(Bytes::from)
        .ok_or(BytesError::InvalidBase32)
}

/**
 * Operations
*/
pub fn hamming(a: &Bytes, b: &Bytes) -> u32 {
    a.iter()
        .zip(b.iter())
        .map(|(&x, &y)| (x ^ y).count_ones())
        .sum()
}

pub fn random(len: usize) -> Bytes {
    let mut rng = rand::thread_rng();
    let mut vec = vec![0u8; len];
    rng.fill(vec.as_mut_slice());
    Bytes::from(vec)
}

pub fn concat(data: &[Bytes]) -> Bytes {
    let total_size = data.iter().map(|d| d.len()).sum();
    let mut vec = Vec::with_capacity(total_size);
    for d in data {
        vec.extend_from_slice(d);
    }
    Bytes::from(vec)
}
