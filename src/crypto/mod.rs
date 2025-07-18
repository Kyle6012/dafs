use aes_gcm::KeyInit;
use aes_gcm::{Aes256Gcm, Key, Nonce};
use aes_gcm::aead::Aead;
use rand::RngCore;
use x25519_dalek::{EphemeralSecret, PublicKey, SharedSecret};
use pbkdf2::pbkdf2_hmac;
use sha2::Sha256;
use typenum::U12;
use generic_array::GenericArray;

pub fn encrypt_file(contents: &[u8], key_bytes: &[u8; 32]) -> anyhow::Result<Vec<u8>> {
    let key = Key::<aes_gcm::aes::Aes256>::from_slice(key_bytes);
    let cipher = Aes256Gcm::new(key);
    let mut nonce_bytes = [0u8; 12];
    rand::thread_rng().fill_bytes(&mut nonce_bytes);
    let nonce = Nonce::from_slice(&nonce_bytes);
    let ciphertext = cipher.encrypt(nonce, contents)
        .map_err(|e| anyhow::Error::msg(format!("AES-GCM encryption failed: {:?}", e)))?;
    Ok([&nonce_bytes[..], &ciphertext[..]].concat())
}

pub fn decrypt_file(ciphertext: &[u8], key_bytes: &[u8; 32]) -> anyhow::Result<Vec<u8>> {
    if ciphertext.len() < 12 {
        return Err(anyhow::anyhow!("Ciphertext too short"));
    }
    let (nonce_bytes, data) = ciphertext.split_at(12);
    let key = Key::<aes_gcm::aes::Aes256>::from_slice(key_bytes);
    let cipher = Aes256Gcm::new(key);
    let nonce: &GenericArray<u8, U12> = Nonce::from_slice(nonce_bytes);
    let plaintext = cipher.decrypt(nonce, data)
        .map_err(|e| anyhow::Error::msg(format!("AES-GCM decryption failed: {:?}", e)))?;
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

pub fn encrypt_and_save_keypair(secret: &EphemeralSecret, path: &str, password: &str) -> anyhow::Result<()> {
    use aes_gcm::Aes256Gcm;
    use aes_gcm::Key;
    use aes_gcm::Nonce;
    use rand::RngCore;
    use std::fs::File;
    use std::io::Write;

    let mut salt = [0u8; 16];
    rand::thread_rng().fill_bytes(&mut salt);
    let mut key = [0u8; 32];
    pbkdf2_hmac::<Sha256>(password.as_bytes(), &salt, 100_000, &mut key);
    let aes_key = Key::<aes_gcm::aes::Aes256>::from_slice(&key);
    let cipher = Aes256Gcm::new(aes_key);
    let mut nonce_bytes = [0u8; 12];
    rand::thread_rng().fill_bytes(&mut nonce_bytes);
    let nonce = Nonce::from_slice(&nonce_bytes);
    // TODO: EphemeralSecret does not expose its bytes; refactor this logic to use a different key type or store a seed.
    let ciphertext = cipher.encrypt(nonce, &b"TODO_ephemeral_secret"[..]).map_err(|e| anyhow::Error::msg(format!("AES-GCM encryption failed: {:?}", e)))?;
    let mut f = File::create(path)?;
    f.write_all(&salt)?;
    f.write_all(&nonce_bytes)?;
    f.write_all(&ciphertext)?;
    Ok(())
}

pub fn load_and_decrypt_keypair(path: &str, password: &str) -> anyhow::Result<EphemeralSecret> {
    use aes_gcm::Aes256Gcm;
    use aes_gcm::Key;
    use aes_gcm::Nonce;
    use std::fs;
    let data = fs::read(path)?;
    if data.len() < 16 + 12 + 32 { return Err(anyhow::anyhow!("Key file too short")); }
    let (salt, rest) = data.split_at(16);
    let (nonce_bytes, ciphertext) = rest.split_at(12);
    let mut key = [0u8; 32];
    pbkdf2_hmac::<Sha256>(password.as_bytes(), salt, 100_000, &mut key);
    let aes_key = Key::<aes_gcm::aes::Aes256>::from_slice(&key);
    let cipher = Aes256Gcm::new(aes_key);
    let nonce: &GenericArray<u8, U12> = Nonce::from_slice(nonce_bytes);
    // TODO: EphemeralSecret does not expose its bytes; refactor this logic to use a different key type or store a seed.
    Ok(EphemeralSecret::random_from_rng(rand::thread_rng()))
}
