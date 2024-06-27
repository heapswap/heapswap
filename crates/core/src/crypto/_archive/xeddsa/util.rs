// SPDX-FileCopyrightText: 2023 Dominik George <nik@naturalnet.de>
//
// SPDX-License-Identifier: Apache-2.0

/// Generate padding bytes for the `hash_i` function
pub(crate) const fn hash_i_padding<const S: usize>(i: u128) -> [u8; S] {
    let mut padding: [u8; S] = [0xffu8; S];

    let slice = (u128::MAX - i).to_le_bytes();
    let mut idx = 0;
    // for loops due to using iterators can't be used in constant functions
    while idx < slice.len() {
        padding[idx] = slice[idx];
        idx += 1
    }

    padding
}
