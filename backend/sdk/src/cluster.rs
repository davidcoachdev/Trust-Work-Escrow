//! Cluster switch, mainnet guard and secure keypair loading (T18).
//!
//! - `parse_cluster` resolves `CLUSTER`/`RPC_CLUSTER` values into an Anchor
//!   `Cluster`, blocking `mainnet` unless an explicit allowlist env var is set.
//! - `check_keypair_permissions` enforces `0600` (or stricter `0400`) on
//!   keypair files so secrets are not world/group readable.
//! - `load_keypair_secure` validates permissions before delegating to
//!   `read_keypair_file` and never logs secret material.

use crate::error::{BackendError, Result};

/// Env vars that, when set to `1`/`true` (case-insensitive), allow mainnet.
///
/// Checked in order `TRUST_ESCROW_ALLOW_MAINNET`, `ALLOW_MAINNET`,
/// `TRUST_ALLOW_MAINNET` so existing `ALLOW_MAINNET=1` setups keep working.
const ALLOW_MAINNET_VARS: &[&str] = &[
    "TRUST_ESCROW_ALLOW_MAINNET",
    "ALLOW_MAINNET",
    "TRUST_ALLOW_MAINNET",
];

/// Returns true if any allowlist env var explicitly enables mainnet.
///
/// Accepted truthy values (case-insensitive, trimmed): `1`, `true`, `yes`, `on`.
pub fn is_mainnet_allowed() -> bool {
    for var in ALLOW_MAINNET_VARS {
        if let Ok(val) = std::env::var(var) {
            let v = val.trim().to_ascii_lowercase();
            if matches!(v.as_str(), "1" | "true" | "yes" | "on") {
                return true;
            }
        }
    }
    false
}

/// Returns true if `s` looks like a mainnet cluster identifier or URL.
///
/// Covers:
/// - bare identifiers: `mainnet`, `mainnet-beta`
/// - URLs containing `mainnet` or `api.mainnet-beta.solana.com`
///
/// Case-insensitive, trimmed.
pub fn is_mainnet_str(s: &str) -> bool {
    let lower = s.trim().to_ascii_lowercase();
    lower == "mainnet"
        || lower == "mainnet-beta"
        || lower.contains("mainnet")
        || lower.contains("api.mainnet-beta.solana.com")
}

/// Check file permissions for a keypair path. On Unix, rejects files whose
/// mode allows group/other read/write/exec (i.e. `mode & 0o077 != 0`).
/// Accepts `0o600` and stricter `0o400`; rejects `0o644`, `0o640`, etc.
///
/// On non-Unix targets this is a no-op that always succeeds, since the Unix
/// permission model does not apply.
///
/// The error message includes the octal mode but **never** the keypair bytes.
pub fn check_keypair_permissions(path: &str) -> Result<()> {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let meta = std::fs::metadata(path).map_err(|e| {
            BackendError::keypair_error(format!("cannot stat keypair file {}: {}", path, e))
        })?;
        let mode = meta.permissions().mode() & 0o777;
        // Reject if group/other have any bits set. 0o600 (rw-------) and 0o400 (r--------)
        // are allowed; anything with group/other bits like 0o640, 0o644, 0o777 is rejected.
        if mode & 0o077 != 0 {
            return Err(BackendError::keypair_error(format!(
                "insecure keypair permissions {:o} for {}: expected 0600 (or 0400), run `chmod 600 {}`",
                mode, path, path
            )));
        }
    }
    // Non-unix: no permission model to enforce.
    #[cfg(not(unix))]
    {
        let _ = path;
    }
    Ok(())
}

/// Load a keypair from `path` after validating permissions. Never logs secret
/// material; errors only mention the path and the underlying I/O/parse error
/// without dumping key bytes.
#[cfg(feature = "solana")]
pub fn load_keypair_secure(path: &str) -> Result<solana_sdk::signature::Keypair> {
    check_keypair_permissions(path)?;
    // Use solana_sdk's helper but map error to not leak bytes beyond path+msg.
    let kp = solana_sdk::signature::read_keypair_file(path)
        .map_err(|e| BackendError::keypair_error(format!("{}: {}", path, e)))?;
    Ok(kp)
}

// ---------------------------------------------------------------------------
// Cluster parsing (requires `solana` feature for `anchor_client::Cluster`)
// ---------------------------------------------------------------------------

#[cfg(feature = "solana")]
mod cluster_impl {
    use super::{is_mainnet_allowed, is_mainnet_str, BackendError, Result};
    use anchor_client::Cluster;

    /// Parse a cluster identifier (env value) into an Anchor [`Cluster`].
    ///
    /// Blocks `mainnet` / `mainnet-beta` and any URL containing `mainnet`
    /// unless an allowlist env var is set (see [`is_mainnet_allowed`]).
    /// Returns a typed `BackendError::Config` on block so CI/tests fail fast.
    pub fn parse_cluster(s: &str) -> Result<Cluster> {
        let trimmed = s.trim();
        if trimmed.is_empty() {
            return Err(BackendError::config_error("cluster value is empty"));
        }
        let lower = trimmed.to_ascii_lowercase();
        match lower.as_str() {
            "localnet" | "localhost" => Ok(Cluster::Localnet),
            "devnet" => Ok(Cluster::Devnet),
            "testnet" => Ok(Cluster::Testnet),
            "mainnet" | "mainnet-beta" => {
                if is_mainnet_allowed() {
                    Ok(Cluster::Mainnet)
                } else {
                    Err(BackendError::config_error(
                        "mainnet blocked: set ALLOW_MAINNET=1 or TRUST_ESCROW_ALLOW_MAINNET=1 to enable",
                    ))
                }
            }
            _ => {
                // Custom URL or unknown identifier. Block if it looks like mainnet.
                if is_mainnet_str(trimmed) && !is_mainnet_allowed() {
                    return Err(BackendError::config_error(
                        "mainnet URL blocked: set ALLOW_MAINNET=1 or TRUST_ESCROW_ALLOW_MAINNET=1 to enable",
                    ));
                }
                Ok(Cluster::Custom(trimmed.to_string(), trimmed.to_string()))
            }
        }
    }

    /// Validate an already-constructed `Cluster` against the mainnet allowlist.
    ///
    /// Call this from `TrustEscrowClient::new` so direct construction also
    /// respects the guard (not just `parse_cluster` via env).
    pub fn validate_cluster(cluster: &Cluster) -> Result<()> {
        let is_mainnet = matches!(cluster, Cluster::Mainnet)
            || matches!(cluster, Cluster::Custom(url, _) if is_mainnet_str(url));
        if is_mainnet && !is_mainnet_allowed() {
            return Err(BackendError::config_error(
                "mainnet blocked: set ALLOW_MAINNET=1 or TRUST_ESCROW_ALLOW_MAINNET=1 to enable",
            ));
        }
        Ok(())
    }
}

#[cfg(feature = "solana")]
pub use cluster_impl::{parse_cluster, validate_cluster};

#[cfg(test)]
mod tests {
    use super::*;
    use serial_test::serial;

    #[test]
    fn mainnet_str_detection() {
        assert!(is_mainnet_str("mainnet"));
        assert!(is_mainnet_str("mainnet-beta"));
        assert!(is_mainnet_str("MAINNET"));
        assert!(is_mainnet_str("https://api.mainnet-beta.solana.com"));
        assert!(is_mainnet_str("https://my-mainnet.example.com/rpc"));
        assert!(!is_mainnet_str("devnet"));
        assert!(!is_mainnet_str("localnet"));
        assert!(!is_mainnet_str("http://127.0.0.1:8899"));
        assert!(!is_mainnet_str("https://api.devnet.solana.com"));
    }

    #[test]
    #[serial]
    fn allowlist_defaults_blocked() {
        // Ensure clean env for this test (serial_test recommended in integration).
        std::env::remove_var("ALLOW_MAINNET");
        std::env::remove_var("TRUST_ESCROW_ALLOW_MAINNET");
        std::env::remove_var("TRUST_ALLOW_MAINNET");
        assert!(!is_mainnet_allowed());
    }

    #[test]
    #[serial]
    fn allowlist_truthy_values() {
        std::env::set_var("ALLOW_MAINNET", "1");
        assert!(is_mainnet_allowed());
        std::env::set_var("ALLOW_MAINNET", "true");
        assert!(is_mainnet_allowed());
        std::env::set_var("ALLOW_MAINNET", "TRUE");
        assert!(is_mainnet_allowed());
        std::env::set_var("ALLOW_MAINNET", "yes");
        assert!(is_mainnet_allowed());
        std::env::remove_var("ALLOW_MAINNET");
        assert!(!is_mainnet_allowed());
    }

    #[cfg(unix)]
    #[test]
    fn keypair_perms_rejects_world_readable() {
        use std::os::unix::fs::PermissionsExt;
        let dir = std::env::temp_dir();
        let path = dir.join(format!("test-kp-perms-{}-644.json", std::process::id()));
        // Minimal valid keypair bytes (64 zeros) as JSON array — read_keypair_file won't be called
        // for perms check; we only test permission rejection before read.
        std::fs::write(&path, "[0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0]").unwrap();
        std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o644)).unwrap();
        let res = check_keypair_permissions(path.to_str().unwrap());
        assert!(res.is_err());
        let msg = res.unwrap_err().to_string();
        assert!(
            msg.contains("insecure keypair permissions"),
            "msg was: {}",
            msg
        );
        assert!(!msg.contains("0,0,0"), "must not leak key bytes");
        std::fs::remove_file(&path).unwrap();
    }

    #[cfg(unix)]
    #[test]
    fn keypair_perms_allows_0600_and_0400() {
        use std::os::unix::fs::PermissionsExt;
        let dir = std::env::temp_dir();
        for mode in [0o600, 0o400] {
            let path = dir.join(format!(
                "test-kp-perms-{}-{:o}.json",
                std::process::id(),
                mode
            ));
            std::fs::write(&path, "[]").unwrap();
            std::fs::set_permissions(&path, std::fs::Permissions::from_mode(mode)).unwrap();
            let res = check_keypair_permissions(path.to_str().unwrap());
            assert!(res.is_ok(), "mode {:o} should be allowed: {:?}", mode, res);
            std::fs::remove_file(&path).unwrap();
        }
    }

    #[cfg(feature = "solana")]
    mod cluster_tests {
        use super::super::*;
        use serial_test::serial;

        #[test]
        #[serial]
        fn parse_cluster_blocks_mainnet_without_allowlist() {
            std::env::remove_var("ALLOW_MAINNET");
            std::env::remove_var("TRUST_ESCROW_ALLOW_MAINNET");
            std::env::remove_var("TRUST_ALLOW_MAINNET");

            let res = parse_cluster("mainnet");
            assert!(res.is_err(), "mainnet should be blocked");
            assert!(res.unwrap_err().to_string().contains("mainnet blocked"));

            let res2 = parse_cluster("mainnet-beta");
            assert!(res2.is_err());

            let res3 = parse_cluster("https://api.mainnet-beta.solana.com");
            assert!(res3.is_err(), "mainnet URL should be blocked: {:?}", res3);

            // Non-mainnet should succeed without allowlist
            assert!(parse_cluster("localnet").is_ok());
            assert!(parse_cluster("devnet").is_ok());
            assert!(parse_cluster("http://127.0.0.1:8899").is_ok());
        }

        #[test]
        #[serial]
        fn parse_cluster_allows_mainnet_with_allowlist() {
            std::env::set_var("ALLOW_MAINNET", "1");
            assert!(parse_cluster("mainnet").is_ok());
            assert!(parse_cluster("mainnet-beta").is_ok());
            assert!(parse_cluster("https://api.mainnet-beta.solana.com").is_ok());
            std::env::remove_var("ALLOW_MAINNET");
        }

        #[test]
        #[serial]
        fn validate_cluster_blocks_direct_mainnet() {
            use anchor_client::Cluster;
            std::env::remove_var("ALLOW_MAINNET");
            std::env::remove_var("TRUST_ESCROW_ALLOW_MAINNET");
            std::env::remove_var("TRUST_ALLOW_MAINNET");

            let c = Cluster::Mainnet;
            assert!(validate_cluster(&c).is_err());

            let c2 = Cluster::Custom(
                "https://api.mainnet-beta.solana.com".to_string(),
                "https://api.mainnet-beta.solana.com".to_string(),
            );
            assert!(validate_cluster(&c2).is_err());

            // Allowed with env
            std::env::set_var("TRUST_ESCROW_ALLOW_MAINNET", "true");
            assert!(validate_cluster(&c).is_ok());
            assert!(validate_cluster(&c2).is_ok());
            std::env::remove_var("TRUST_ESCROW_ALLOW_MAINNET");
        }
    }
}
