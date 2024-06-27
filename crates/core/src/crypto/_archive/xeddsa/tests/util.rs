// SPDX-FileCopyrightText: 2023 Dominik George <nik@naturalnet.de>
//
// SPDX-License-Identifier: Apache-2.0

use crate::crypto::xeddsa::util::*;

#[test]
fn test_hash_i_padding_1() {
    let padding: [u8; 32] = hash_i_padding(1);
    assert_eq!(padding[0], 0xfe);
    assert_eq!(padding[1..32], [0xffu8; 31]);
}
