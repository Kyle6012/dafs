use aes_gcm::KeyInit;
use aes_gcm::{Aes256Gcm, Key, Nonce};
use aes_gcm::aead::Aead;
use rand::RngCore;
use x25519_dalek::{EphemeralSecret, PublicKey, SharedSecret};

pub fn encrypt_file(contents: &[u8], key_bytes: &[u8; 32]) -> anyhow::Result<Vec<u8>> {
    let key = Key::<aes_gcm::aes::Aes256>::from_slice(key_bytes);
    let cipher = Aes256Gcm::new(key);
    let mut nonce_bytes = [0u8; 12];
    rand::thread_rng().fill_bytes(&mut nonce_bytes);
    let nonce = Nonce::from_slice(&nonce_bytes);
    let ciphertext = cipher.encrypt(nonce, contents)
        .map_err(|e| anyhow::anyhow!("AES-GCM encryption failed: {:?}", e))?;
    Ok([&nonce_bytes[..], &ciphertext[..]].concat())
}

pub fn decrypt_file(ciphertext: &[u8], key_bytes: &[u8; 32]) -> anyhow::Result<Vec<u8>> {
    if ciphertext.len() < 12 {
        return Err(anyhow::anyhow!("Ciphertext too short"));
    }
    let (nonce_bytes, data) = ciphertext.split_at(12);
    let key = Key::<aes_gcm::aes::Aes256>::from_slice(key_bytes);
    let cipher = Aes256Gcm::new(key);
    let nonce = Nonce::from_slice(nonce_bytes);
    let plaintext = cipher.decrypt(nonce, data)
        .map_err(|e| anyhow::anyhow!("AES-GCM decryption failed: {:?}", e))?;
    Ok(plaintext)
}

pub fn generate_x25519_keypair() -> (EphemeralSecret, PublicKey) {
    let secret = EphemeralSecret::random_from_rng(rand::thread_rng());
    let public = PublicKey::from(&secret);
    (secret, public)
}

pub fn derive_shared_secret(secret: EphemeralSecret, peer_public: &PublicKey) -> SharedSecret {
    secret.diffie_hellman(peer_public)
}
