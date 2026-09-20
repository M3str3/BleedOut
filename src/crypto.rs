use base64;
use rand::rngs::OsRng;
use rsa::pkcs8::{DecodePrivateKey, DecodePublicKey};
use rsa::{Oaep, RsaPrivateKey, RsaPublicKey};
use sha2::Sha256;

/// Encrypt a raw AES key using the RSA public key (OAEP/SHA-256).
pub fn encrypt_aes_key(aes_key: &[u8], public_pem: &str) -> Result<Vec<u8>, String> {
    let public_key = RsaPublicKey::from_public_key_pem(public_pem)
        .map_err(|e| format!("Invalid public key: {e}"))?;
    let mut rng = OsRng;
    let padding = Oaep::new::<Sha256>();
    public_key
        .encrypt(&mut rng, padding, aes_key)
        .map_err(|e| format!("RSA encrypt failed: {e}"))
}

/// Decrypt an AES key blob using the RSA private key (PKCS#8 PEM).
pub fn decrypt_aes_key(encrypted_aes_key: &[u8], private_pem: &str) -> Result<Vec<u8>, String> {
    let private_key = RsaPrivateKey::from_pkcs8_pem(private_pem)
        .map_err(|e| format!("Invalid private key: {e}"))?;
    let padding = Oaep::new::<Sha256>();
    private_key
        .decrypt(padding, encrypted_aes_key)
        .map_err(|e| format!("RSA decrypt failed: {e}"))
}

pub fn base64_encode(data: &[u8]) -> String {
    base64::encode(data)
}

pub fn base64_decode(s: &str) -> Result<Vec<u8>, String> {
    base64::decode(s.trim()).map_err(|e| format!("Base64 decode failed: {e}"))
}
