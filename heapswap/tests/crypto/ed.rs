use rand::Rng;

#[test]
pub fn test_ed_sign(){
  
  use rand::rngs::OsRng;
  use ed25519_dalek::{SigningKey, Signature, Signer};  
  use ed25519_dalek::{PUBLIC_KEY_LENGTH, SECRET_KEY_LENGTH, KEYPAIR_LENGTH, SIGNATURE_LENGTH};
  use ed25519_dalek::{VerifyingKey, Verifier};
  
  // the dalek library calls the private key a signing key and the public key a verifying key

  // get the random number generator
  let mut rng = OsRng;
  
  // generate a random private key
  let private_key_bytes: [u8; SECRET_KEY_LENGTH] = rng.gen();
  
  // the private_key object is just a wrapper around the private_key_bytes
  let private_key: SigningKey = SigningKey::from_bytes(&private_key_bytes);
  assert_eq!(private_key_bytes, private_key.to_bytes());
  
  // sign a message
  let message: &[u8] = b"This is a test of the tsunami alert system.";
  let signature: Signature = private_key.sign(message);
  
  // verify the signature
  assert!(private_key.verify(message, &signature).is_ok());

  // get the public key
  let public_key: VerifyingKey = private_key.verifying_key();
  
  // verify the signature with the public key
  assert!(public_key.verify(message, &signature).is_ok());

  /*
  // serialize
  // they all have the .to_bytes() method, but there is also a .to_keypair_bytes() method that returns a [u8; KEYPAIR_LENGTH]
  let public_key_bytes: [u8; PUBLIC_KEY_LENGTH] = private_key.verifying_key().to_bytes();
  let private_key_bytes: [u8; SECRET_KEY_LENGTH] = private_key.to_bytes();
  let keypair_bytes:    [u8; KEYPAIR_LENGTH]    = private_key.to_keypair_bytes();
  let signature_bytes:  [u8; SIGNATURE_LENGTH]  = signature.to_bytes();
  */
  /*
  // deserialize
  // the from_bytes() method returns a Result, so you have to unwrap it
  let public_key: VerifyingKey = VerifyingKey::from_bytes(&public_key_bytes).unwrap();
  let private_key: SigningKey = SigningKey::from_bytes(&private_key_bytes);
  let signature: Signature = Signature::try_from(&signature_bytes[..]).unwrap();
  */
}

#[test]
pub fn test_ed_exchange(){
  use rand::rngs::OsRng;
  use x25519_dalek::{EphemeralSecret, PublicKey};

  // get the random number generator
  let mut rng = OsRng;

  // generate alice's secret key
  let alice_private = EphemeralSecret::random_from_rng(rng);
  let alice_public = PublicKey::from(&alice_private);
  
  // generate bob's secret key
  let bob_private = EphemeralSecret::random_from_rng(rng);
  let bob_public = PublicKey::from(&bob_private);
  
  // calculate the shared secret
  let alice_shared_secret = alice_private.diffie_hellman(&bob_public);
  let bob_shared_secret = bob_private.diffie_hellman(&alice_public);
  // the shared secret is the same for both alice and bob
  assert_eq!(alice_shared_secret.as_bytes(), bob_shared_secret.as_bytes());
  
  // send a message from alice to bob
  use chacha20poly1305::{
    aead::{Aead, AeadCore, KeyInit, generic_array::GenericArray},
    ChaCha20Poly1305, Nonce
  };
  
  // generate a random nonce
  let nonce = ChaCha20Poly1305::generate_nonce(rng); // 96-bits; unique per message
  
  // encrypt the message
  let alice_shared_secret_bytes = alice_shared_secret.as_bytes();
  // Use the first 32 bytes of the shared secret as the key
  let alice_key = GenericArray::clone_from_slice(&alice_shared_secret_bytes[..32]);
  let message = b"Hello Bob!";
  let alice_cipher = ChaCha20Poly1305::new(&alice_key);
  let ciphertext = alice_cipher.encrypt(&nonce, message.as_ref()).unwrap();
  
  // alice sends the ciphertext and the nonce to bob
  let bob_shared_secret_bytes = bob_shared_secret.as_bytes();
  let bob_shared_key = GenericArray::clone_from_slice(&bob_shared_secret_bytes[..32]);
  let bob_cipher = ChaCha20Poly1305::new(&bob_shared_key);
  let plaintext = bob_cipher.decrypt(&nonce, ciphertext.as_ref()).unwrap();
  // bob successfully decrypts the message
  assert_eq!(&plaintext, message);

}

