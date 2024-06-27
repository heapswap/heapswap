use crate::{crypto::keys::public_key, traits::Randomable};

use super::{
	KeyPair, PublicKey, PrivateKey
};


const TEST_MSG: &[u8] = b"Hello, world!";

#[test]
fn test_keys(){
	
	
	let private_key = PrivateKey::from_random();
    //let public_key = PublicKey::from(&x25519_dalek::PublicKey::from(
    //    &x25519_dalek::StaticSecret::from(PRIV_IN),
    //));
	let public_key = PublicKey::from_private_key(&private_key);

    let signature: [u8; 64] = private_key.sign(TEST_MSG, OsRng);
    let valid = public_key.verify(TEST_MSG, &signature);

    assert!(valid.is_ok());
	
}