// convert 256 bits between u8*32 and u64*4
// this is used to speed up comparisons of public keys

type Unstacked = [u8; 32];
type Stacked = [u64; 4];

pub fn stack_256(key: &Unstacked) -> Stacked {
    let mut stacked = [0u64; 4];
    for i in 0..4 {
        stacked[i] = u64::from_le_bytes(key[i * 8..(i + 1) * 8].try_into().unwrap());
    }
    stacked
}

pub fn unstack_256(stacked: &Stacked) -> Unstacked {
    let mut key = [0u8; 32];
    for i in 0..4 {
        key[i * 8..(i + 1) * 8].copy_from_slice(&stacked[i].to_le_bytes());
    }
    key
}
