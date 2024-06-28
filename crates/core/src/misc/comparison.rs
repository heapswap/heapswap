use num_traits::PrimInt;
use std::ops::BitXor;

// Generic XOR function for arrays of any size and integer type
pub fn xor<T, const N: usize>(a: &[T; N], b: &[T; N]) -> [T; N]
where
    T: PrimInt + BitXor<Output = T>, // Ensure T supports XOR and is a primitive integer
{
    let mut result = [T::zero(); N]; // Initialize array with zeros
    for i in 0..N {
        result[i] = a[i] ^ b[i];
    }
    result
}

// Generic Hamming distance function for arrays of any size and integer type
pub fn hamming<T, const N: usize>(a: &[T; N], b: &[T; N]) -> u32
where
    T: PrimInt + BitXor<Output = T>, // Ensure T supports XOR and is a primitive integer
{
    let mut result = 0u32;
    for i in 0..N {
        result += (a[i] ^ b[i]).count_ones();
    }
    result
}
