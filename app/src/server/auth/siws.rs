//! SIWS — Sign-In With Solana, free con `ed25519-dalek` + `bs58`
//! Verifica que `pubkey` firmó `message` con `signature` (todo base58).

#[cfg(feature = "server")]
use ed25519_dalek::{Signature, VerifyingKey};

#[cfg(feature = "server")]
pub fn verify_siws(pubkey_b58: &str, message: &str, signature_b58: &str) -> Result<bool, String> {
    let pubkey_bytes = bs58::decode(pubkey_b58)
        .into_vec()
        .map_err(|e| format!("pubkey base58: {:?}", e))?;
    if pubkey_bytes.len() != 32 {
        return Err("pubkey debe ser 32 bytes".to_string());
    }
    let mut pk_arr = [0u8; 32];
    pk_arr.copy_from_slice(&pubkey_bytes);
    let vk = VerifyingKey::from_bytes(&pk_arr).map_err(|e| format!("pubkey inválido: {:?}", e))?;

    let sig_bytes = bs58::decode(signature_b58)
        .into_vec()
        .map_err(|e| format!("signature base58: {:?}", e))?;
    if sig_bytes.len() != 64 {
        return Err("signature debe ser 64 bytes".to_string());
    }
    let mut sig_arr = [0u8; 64];
    sig_arr.copy_from_slice(&sig_bytes);
    let sig = Signature::from_bytes(&sig_arr);

    Ok(vk.verify_strict(message.as_bytes(), &sig).is_ok())
}

// Server function — llamada desde Dioxus WASM (Phantom firma)
use dioxus::prelude::*;

#[server]
pub async fn verify_siws_server(
    pubkey: String,
    message: String,
    signature: String,
) -> Result<String, ServerFnError> {
    // En server, verificar con ed25519
    #[cfg(feature = "server")]
    {
        match verify_siws(&pubkey, &message, &signature) {
            Ok(true) => Ok("verified".to_string()),
            Ok(false) => Err(ServerFnError::new("firma inválida".to_string())),
            Err(e) => Err(ServerFnError::new(e)),
        }
    }
    #[cfg(not(feature = "server"))]
    {
        Err(ServerFnError::new("server only".to_string()))
    }
}

#[cfg(all(test, feature = "server"))]
mod tests {
    use super::*;
    use ed25519_dalek::{SigningKey, Signer};
    use rand::rngs::OsRng;

    #[test]
    fn siws_verify_ok() {
        let mut csprng = OsRng;
        let sk = SigningKey::generate(&mut csprng);
        let vk = sk.verifying_key();
        let msg = "Link wallet to guest 123";
        let sig = sk.sign(msg.as_bytes());
        let pubkey_b58 = bs58::encode(vk.to_bytes()).into_string();
        let sig_b58 = bs58::encode(sig.to_bytes()).into_string();
        assert!(verify_siws(&pubkey_b58, msg, &sig_b58).unwrap());
    }

    #[test]
    fn siws_verify_wrong_msg() {
        let mut csprng = OsRng;
        let sk = SigningKey::generate(&mut csprng);
        let vk = sk.verifying_key();
        let sig = sk.sign(b"hello");
        let pubkey_b58 = bs58::encode(vk.to_bytes()).into_string();
        let sig_b58 = bs58::encode(sig.to_bytes()).into_string();
        assert!(!verify_siws(&pubkey_b58, "wrong", &sig_b58).unwrap());
    }
}
