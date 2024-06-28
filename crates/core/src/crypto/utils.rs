// convert 256 bits between u8*32 and u64*4
// this is used to speed up comparisons of public keys

//type Unpacked = [u8; 32];
//type Packed = [u64; 4];

//pub fn pack_256(key: &Unpacked) -> Packed {
//    let mut packed = [0u64; 4];
//    for i in 0..4 {
//        packed[i] = u64::from_le_bytes(key[i * 8..(i + 1) * 8].try_into().unwrap());
//    }
//    packed
//}

//pub fn unpack_256(packed: &Packed) -> Unpacked {
//    let mut key = [0u8; 32];
//    for i in 0..4 {
//        key[i * 8..(i + 1) * 8].copy_from_slice(&packed[i].to_le_bytes());
//    }
//    key
//}
