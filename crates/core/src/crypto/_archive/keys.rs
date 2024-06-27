use crate::bys;
use bytes::Bytes;
use ed25519_dalek::{
    Signature as DalekSignature, Signer, SigningKey, Verifier, VerifyingKey, PUBLIC_KEY_LENGTH,
    SECRET_KEY_LENGTH, SIGNATURE_LENGTH,
};
use rand::rngs::OsRng;
use rand::Rng;
use serde::{Deserialize, Serialize};
use x25519_dalek::{EphemeralSecret, PublicKey as X25519PublicKey, StaticSecret};

#[derive(Debug)]
pub enum KeyPairError {
    InvalidPublicKey,
    InvalidPrivateKey,
    InvalidSharedKey,
    InvalidKeyPair,
}

#[derive(Clone, Serialize, Deserialize, PartialEq, Debug)]
pub struct SharedKey([u8; 32]);

impl SharedKey {
    pub fn to_bytes(&self) -> Bytes {
        self.0.to_vec().into()
    }

    pub fn from_bytes(bytes: &Bytes) -> Result<SharedKey, KeyPairError> {
        let bytes_array: &[u8] = bytes.as_ref();
        if bytes_array.len() == 32 {
            let mut array = [0u8; 32];
            array.copy_from_slice(bytes_array);
            Ok(SharedKey(array))
        } else {
            Err(KeyPairError::InvalidSharedKey)
        }
    }

    pub fn to_string(&self) -> String {
        bys::to_base32(&self.to_bytes())
    }

    pub fn from_string(data: &String) -> Result<SharedKey, KeyPairError> {
        SharedKey::from_bytes(&bys::from_base32(data).map_err(|_| KeyPairError::InvalidSharedKey)?)
    }
}

#[derive(Clone, Serialize, Deserialize, PartialEq, Debug)]
pub struct PublicKey(VerifyingKey);

impl PublicKey {
    pub fn to_string(&self) -> String {
        bys::to_base32(&self.to_bytes())
    }

    pub fn from_string(data: &String) -> Result<PublicKey, KeyPairError> {
        PublicKey::from_bytes(&bys::from_base32(data).map_err(|_| KeyPairError::InvalidPublicKey)?)
    }

    pub fn to_bytes(&self) -> Bytes {
        self.0.to_bytes().to_vec().into()
    }

    pub fn from_bytes(bytes: &Bytes) -> Result<PublicKey, KeyPairError> {
        // Attempt to convert &Bytes to &[u8; 32]
        let bytes_array: &[u8] = bytes.as_ref();
        if bytes_array.len() == PUBLIC_KEY_LENGTH {
            let mut array = [0u8; PUBLIC_KEY_LENGTH];
            array.copy_from_slice(bytes_array);
            VerifyingKey::from_bytes(&array)
                .map(PublicKey)
                .map_err(|_| KeyPairError::InvalidPublicKey)
        } else {
            Err(KeyPairError::InvalidPublicKey)
        }
    }

    pub fn verify(&self, data: &Bytes, signature: &Signature) -> bool {
        self.0.verify(data.as_ref(), &signature.0).is_ok()
    }
}

#[derive(Clone, Serialize, Deserialize, PartialEq, Debug)]
pub struct PrivateKey(SigningKey);

impl PrivateKey {
    pub fn to_string(&self) -> String {
        bys::to_base32(&self.to_bytes())
    }

    pub fn from_string(data: &String) -> Result<PrivateKey, KeyPairError> {
        PrivateKey::from_bytes(
            &bys::from_base32(data).map_err(|_| KeyPairError::InvalidPrivateKey)?,
        )
    }

    pub fn to_bytes(&self) -> Bytes {
        self.0.to_bytes().to_vec().into()
    }

    pub fn from_bytes(bytes: &Bytes) -> Result<PrivateKey, KeyPairError> {
        let bytes_array: &[u8] = bytes.as_ref();
        if bytes_array.len() == SECRET_KEY_LENGTH {
            let mut array = [0u8; SECRET_KEY_LENGTH];
            array.copy_from_slice(bytes_array);
            Ok(PrivateKey(SigningKey::from_bytes(&array)))
        } else {
            Err(KeyPairError::InvalidPrivateKey)
        }
    }

    pub fn sign(&self, data: &Bytes) -> Signature {
        Signature(self.0.sign(data.as_ref()))
    }

    pub fn shared(&self, public_key: &PublicKey) -> SharedKey {
        let private_montgomery = StaticSecret::from(
            curve25519_dalek::edwards::CompressedEdwardsY::from_slice(&self.0.to_bytes())
                .unwrap()
                .decompress()
                .unwrap()
                .to_montgomery()
                .to_bytes(),
        );
        let public_montgomery = X25519PublicKey::from(
            curve25519_dalek::edwards::CompressedEdwardsY::from_slice(&public_key.0.to_bytes())
                .unwrap()
                .decompress()
                .unwrap()
                .to_montgomery()
                .to_bytes(),
        );

        let shared_secret = private_montgomery.diffie_hellman(&public_montgomery);
        shared_secret.to_bytes();

        return SharedKey(shared_secret.to_bytes());
    }
}

#[derive(Clone, Serialize, Deserialize, PartialEq, Debug)]
pub struct KeyPair {
    public_key: PublicKey,
    private_key: PrivateKey,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Signature(DalekSignature);

impl Signature {
    pub fn to_string(&self) -> String {
        bys::to_base32(&self.to_bytes())
    }

    pub fn from_string(data: &String) -> Result<Signature, KeyPairError> {
        Signature::from_bytes(&bys::from_base32(data).map_err(|_| KeyPairError::InvalidPrivateKey)?)
    }

    pub fn to_bytes(&self) -> Bytes {
        self.0.to_bytes().to_vec().into()
    }

    pub fn from_bytes(bytes: &Bytes) -> Result<Signature, KeyPairError> {
        let bytes_array: &[u8] = bytes.as_ref();
        if bytes_array.len() == SIGNATURE_LENGTH {
            let mut array = [0u8; SIGNATURE_LENGTH];
            array.copy_from_slice(bytes_array);
            Ok(Signature(DalekSignature::from_bytes(&array)))
        } else {
            Err(KeyPairError::InvalidPrivateKey)
        }
    }
}

impl KeyPair {
    /**
     * Instantiation
    	*/
    pub fn new() -> Result<KeyPair, KeyPairError> {
        let mut random_bytes: [u8; SECRET_KEY_LENGTH] = [0; SECRET_KEY_LENGTH];
        OsRng.fill(&mut random_bytes);
        let private_key: PrivateKey = PrivateKey(SigningKey::from_bytes(&random_bytes));
        let public_key = PublicKey(VerifyingKey::from(&private_key.0));

        Ok(KeyPair {
            public_key,
            private_key,
        })
    }

    pub fn to_string(&self) -> String {
        bys::to_base32(&self.to_bytes())
    }

    pub fn from_string(data: &String) -> Result<KeyPair, KeyPairError> {
        KeyPair::from_bytes(&bys::from_base32(data).map_err(|_| KeyPairError::InvalidKeyPair)?)
    }

    pub fn from_bytes(keypair: &Bytes) -> Result<KeyPair, KeyPairError> {
        let public_key = PublicKey::from_bytes(&keypair.slice(0..PUBLIC_KEY_LENGTH))?;
        let private_key = PrivateKey::from_bytes(&keypair.slice(PUBLIC_KEY_LENGTH..))?;
        Ok(KeyPair {
            public_key,
            private_key,
        })
    }

    pub fn to_bytes(&self) -> Bytes {
        bys::concat(&[self.public_key.to_bytes(), self.private_key.to_bytes()])
    }

    /**
     * Operations
    	*/
    pub fn public_key(&self) -> PublicKey {
        self.public_key.clone()
    }

    pub fn private_key(&self) -> PrivateKey {
        self.private_key.clone()
    }

    pub fn sign(&self, data: &Bytes) -> Signature {
        self.private_key.sign(data)
    }

    pub fn verify(&self, data: &Bytes, signature: &Signature) -> bool {
        self.public_key.verify(data, signature)
    }

    pub fn shared(&self, public_key: &PublicKey) -> SharedKey {
        self.private_key.shared(public_key)
    }
}

/*
#[test]
fn test_keypair() -> Result<(), ()> {
    // instantiation
    let keypair = KeyPair::new().unwrap();
    let keypair_bytes = keypair.to_bytes();
    let keypair2 = KeyPair::from_bytes(&keypair_bytes).unwrap();
    assert_eq!(keypair.public_key(), keypair2.public_key());
    assert_eq!(keypair.private_key(), keypair2.private_key());

    println!("public:  {}", &keypair.public_key().to_string());
    println!("private: {}", &keypair.private_key().to_string());
    println!("keypair: {}", &keypair.to_string());

    // sign and verify
    let data = Bytes::from("hello world");
    let signature = keypair.sign(&data);
    assert!(keypair.verify(&data, &signature));

    println!("signature: {}", &signature.to_string());

    // shared key
    let alice = KeyPair::new().unwrap();
    let bob = KeyPair::new().unwrap();

    let shared_key_alice = alice.shared(&bob.public_key());
    let shared_key_bob = bob.shared(&alice.public_key());

    assert_eq!(shared_key_alice, shared_key_bob);

    Ok(())
}
*/