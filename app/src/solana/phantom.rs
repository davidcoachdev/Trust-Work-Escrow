use base64::{engine::general_purpose::STANDARD as BASE64, Engine};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(untagged)]
enum BridgePayload {
    Success { ok: bool, value: String },
    Failure { ok: bool, error: String },
}

#[derive(Clone, Debug, PartialEq)]
pub enum BridgeResponse {
    Value(String),
    Error(String),
}

pub fn parse_bridge_response(json: &str) -> Result<BridgeResponse, String> {
    match serde_json::from_str::<BridgePayload>(json)
        .map_err(|_| "invalid Phantom response".to_string())?
    {
        BridgePayload::Success { ok: true, value } if !value.is_empty() => {
            Ok(BridgeResponse::Value(value))
        }
        BridgePayload::Failure { ok: false, error } if !error.is_empty() => {
            Ok(BridgeResponse::Error(safe_error(&error)))
        }
        _ => Err("invalid Phantom response".to_string()),
    }
}

pub fn validate_base58_public_key(value: &str) -> Result<String, String> {
    if decode_base58(value)
        .map(|bytes| bytes.len() == 32)
        .unwrap_or(false)
    {
        Ok(value.to_string())
    } else {
        Err("Phantom returned an invalid public key".to_string())
    }
}

fn decode_base58(value: &str) -> Result<Vec<u8>, ()> {
    let alphabet = b"123456789ABCDEFGHJKLMNPQRSTUVWXYZabcdefghijkmnopqrstuvwxyz";
    let mut bytes = vec![0u8];
    let leading_zeroes = value
        .bytes()
        .take_while(|character| *character == b'1')
        .count();
    for character in value.bytes().skip(leading_zeroes) {
        let digit = alphabet
            .iter()
            .position(|item| *item == character)
            .ok_or(())? as u32;
        let mut carry = digit;
        for byte in bytes.iter_mut().rev() {
            let current = (*byte as u32) * 58 + carry;
            *byte = current as u8;
            carry = current >> 8;
        }
        while carry > 0 {
            bytes.insert(0, (carry & 0xff) as u8);
            carry >>= 8;
        }
    }
    if bytes == [0] {
        bytes.clear();
    }
    for _ in 0..leading_zeroes {
        bytes.insert(0, 0);
    }
    Ok(bytes)
}

fn safe_error(error: &str) -> String {
    let lower = error.to_ascii_lowercase();
    if lower.contains("reject") || lower.contains("cancel") || lower.contains("denied") {
        "Phantom request rejected by the user".to_string()
    } else {
        "Phantom request failed".to_string()
    }
}

#[cfg(target_arch = "wasm32")]
async fn eval_bridge(script: &str) -> Result<String, String> {
    let mut eval = dioxus::document::eval(script);
    let response: BridgePayload = eval
        .recv()
        .await
        .map_err(|_| "Phantom bridge failed".to_string())?;
    let json = serde_json::to_string(&response).map_err(|_| "Phantom bridge failed".to_string())?;
    match parse_bridge_response(&json)? {
        BridgeResponse::Value(value) => Ok(value),
        BridgeResponse::Error(error) => Err(error),
    }
}

#[cfg(target_arch = "wasm32")]
pub async fn connect() -> Result<String, String> {
    let public_key = eval_bridge(r#"
        const provider = window.solana;
        if (!provider || !provider.isPhantom) {
            dioxus.send({ok: false, error: "Phantom provider not found"});
        } else {
            try {
                const response = await provider.connect();
                const value = response.publicKey?.toString();
                dioxus.send(value ? {ok: true, value} : {ok: false, error: "Phantom returned no public key"});
            } catch (error) {
                dioxus.send({ok: false, error: error?.message || "Phantom connect failed"});
            }
        }
    "#).await?;
    validate_base58_public_key(&public_key)
}

#[cfg(target_arch = "wasm32")]
pub async fn sign_message(message: &str) -> Result<String, String> {
    let mut eval = dioxus::document::eval(
        r#"
        const message = await dioxus.recv();
        const provider = window.solana;
        const encode = bytes => {
            const alphabet = "123456789ABCDEFGHJKLMNPQRSTUVWXYZabcdefghijkmnopqrstuvwxyz";
            let digits = [0];
            for (const byte of bytes) {
                let carry = byte;
                for (let i = digits.length - 1; i >= 0; i--) { const n = digits[i] * 256 + carry; digits[i] = n % 58; carry = Math.floor(n / 58); }
                while (carry) { digits.unshift(carry % 58); carry = Math.floor(carry / 58); }
            }
            return "1".repeat([...bytes].findIndex(byte => byte !== 0) < 0 ? bytes.length : [...bytes].findIndex(byte => byte !== 0)) + digits.reverse().map(d => alphabet[d]).join("");
        };
        if (!provider || !provider.isPhantom) dioxus.send({ok: false, error: "Phantom provider not found"});
        else try {
            const signed = await provider.signMessage(new TextEncoder().encode(message), "utf8");
            const bytes = signed?.signature || signed;
            dioxus.send(bytes ? {ok: true, value: encode(bytes)} : {ok: false, error: "Phantom returned no signature"});
        } catch (error) { dioxus.send({ok: false, error: error?.message || "Phantom sign failed"}); }
    "#,
    );
    eval.send(message)
        .map_err(|_| "Phantom bridge failed".to_string())?;
    let response: BridgePayload = eval
        .recv()
        .await
        .map_err(|_| "Phantom bridge failed".to_string())?;
    let json = serde_json::to_string(&response).map_err(|_| "Phantom bridge failed".to_string())?;
    match parse_bridge_response(&json)? {
        BridgeResponse::Value(value) => Ok(value),
        BridgeResponse::Error(error) => Err(error),
    }
}

#[cfg(target_arch = "wasm32")]
pub async fn sign_transaction(unsigned_base64: &str) -> Result<String, String> {
    let unsigned_base64 = validate_unsigned_transaction(unsigned_base64)?;
    let mut eval = dioxus::document::eval(
        r#"
        const input = await dioxus.recv();
        const provider = window.solana;
        const raw = Uint8Array.from(atob(input), c => c.charCodeAt(0));
        const adapter = globalThis.solanaWeb3;
        if (!provider || !provider.isPhantom) dioxus.send({ok: false, error: "Phantom provider not found"});
        else if (!adapter?.Transaction?.from) dioxus.send({ok: false, error: "Solana transaction adapter unavailable"});
        else try {
            const transaction = adapter.Transaction.from(raw);
            const signed = await provider.signTransaction(transaction);
            if (!signed?.serialize) throw new Error("Phantom returned no signed transaction");
            const bytes = signed.serialize({requireAllSignatures: false, verifySignatures: false});
            let binary = "";
            for (const byte of bytes) binary += String.fromCharCode(byte);
            dioxus.send({ok: true, value: btoa(binary)});
        } catch (error) { dioxus.send({ok: false, error: error?.message || "Phantom transaction signing failed"}); }
    "#,
    );
    eval.send(unsigned_base64)
        .map_err(|_| "Phantom bridge failed".to_string())?;
    let response: BridgePayload = eval
        .recv()
        .await
        .map_err(|_| "Phantom bridge failed".to_string())?;
    let json = serde_json::to_string(&response).map_err(|_| "Phantom bridge failed".to_string())?;
    match parse_bridge_response(&json)? {
        BridgeResponse::Value(value) => Ok(value),
        BridgeResponse::Error(error) => Err(error),
    }
}

#[cfg(target_arch = "wasm32")]
pub async fn get_balance(pubkey: &str) -> Result<u64, String> {
    let pubkey = validate_base58_public_key(pubkey)?;
    let mut eval = dioxus::document::eval(
        r#"
        const pubkey = await dioxus.recv();
        const provider = window.solana;
        if (!provider || !provider.isPhantom) dioxus.send({ok: false, error: "Phantom provider not found"});
        else {
            // Try connection via Phantom's connection if available, else via solanaWeb3
            const conn = provider.connection || (globalThis.solanaWeb3 && new globalThis.solanaWeb3.Connection("https://api.devnet.solana.com"));
            if (!conn || !conn.getBalance) dioxus.send({ok: false, error: "No connection for getBalance"});
            else try {
                const pk = new (globalThis.solanaWeb3 || {}).PublicKey(pubkey);
                // fallback if PublicKey not available, try provider publicKey
                const target = pk || pubkey;
                const lamports = await conn.getBalance(typeof target === 'string' ? new (globalThis.solanaWeb3.PublicKey)(target) : target);
                dioxus.send({ok: true, value: String(lamports)});
            } catch (e) { dioxus.send({ok: false, error: e?.message || "getBalance failed"}); }
        }
    "#,
    );
    eval.send(pubkey.clone()).map_err(|_| "Phantom bridge failed".to_string())?;
    let response: BridgePayload = eval.recv().await.map_err(|_| "Phantom bridge failed".to_string())?;
    let json = serde_json::to_string(&response).map_err(|_| "Phantom bridge failed".to_string())?;
    match parse_bridge_response(&json)? {
        BridgeResponse::Value(v) => v.parse::<u64>().map_err(|_| "invalid balance".to_string()),
        BridgeResponse::Error(e) => Err(e),
    }
}

#[cfg(not(target_arch = "wasm32"))]
pub async fn get_balance(_pubkey: &str) -> Result<u64, String> {
    Err("Phantom is available only in the browser".to_string())
}

#[cfg(not(target_arch = "wasm32"))]
pub async fn connect() -> Result<String, String> {
    Err("Phantom is available only in the browser".to_string())
}

#[cfg(not(target_arch = "wasm32"))]
pub async fn sign_message(_: &str) -> Result<String, String> {
    Err("Phantom is available only in the browser".to_string())
}

#[cfg(not(target_arch = "wasm32"))]
pub async fn sign_transaction(_: &str) -> Result<String, String> {
    Err("Phantom is available only in the browser".to_string())
}

pub fn validate_signed_transaction(value: &str) -> Result<String, String> {
    let bytes = BASE64
        .decode(value)
        .map_err(|_| "Phantom returned invalid transaction bytes".to_string())?;
    if bytes.is_empty() {
        return Err("Phantom returned empty transaction bytes".to_string());
    }
    Ok(BASE64.encode(bytes))
}

pub fn validate_unsigned_transaction(value: &str) -> Result<String, String> {
    let bytes = BASE64
        .decode(value)
        .map_err(|_| "Backend returned invalid unsigned transaction bytes".to_string())?;
    if bytes.is_empty() {
        return Err("Backend returned empty unsigned transaction bytes".to_string());
    }
    Ok(BASE64.encode(bytes))
}

#[cfg(test)]
mod tests {
    use super::{
        parse_bridge_response, validate_base58_public_key, validate_signed_transaction,
        validate_unsigned_transaction, BridgeResponse,
    };

    #[test]
    fn accepts_a_valid_solana_public_key() {
        let key = "11111111111111111111111111111111";
        assert_eq!(validate_base58_public_key(&key).unwrap(), key);
    }

    #[test]
    fn rejects_malformed_bridge_responses() {
        assert!(parse_bridge_response("not-json").is_err());
        assert!(parse_bridge_response(r#"{"ok":true,"value":""}"#).is_err());
    }

    #[test]
    fn parses_safe_bridge_errors() {
        let response = parse_bridge_response(r#"{"ok":false,"error":"User rejected"}"#).unwrap();
        assert_eq!(
            response,
            BridgeResponse::Error("Phantom request rejected by the user".into())
        );
    }

    #[test]
    fn validates_backend_transaction_encoding() {
        assert_eq!(validate_signed_transaction("AQID").unwrap(), "AQID");
        assert!(validate_signed_transaction("not-base64").is_err());
    }

    #[test]
    fn validates_unsigned_transaction_encoding() {
        assert_eq!(validate_unsigned_transaction("AQID").unwrap(), "AQID");
        assert!(validate_unsigned_transaction("").is_err());
        assert!(validate_unsigned_transaction("not-base64").is_err());
    }
}
