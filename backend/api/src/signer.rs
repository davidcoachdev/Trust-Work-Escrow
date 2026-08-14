use std::fmt;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SignerMode {
    ReadOnly,
    UserSigned,
    ServerSigned,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SignerPolicy {
    pub mode: SignerMode,
    pub expected_signer: String,
    pub actor_subject: String,
}

impl SignerPolicy {
    pub fn new(mode: SignerMode, expected_signer: impl Into<String>, actor_subject: impl Into<String>) -> Self {
        Self { mode, expected_signer: expected_signer.into(), actor_subject: actor_subject.into() }
    }

    pub fn validate(&self, signer: Option<&str>) -> Result<(), SignerError> {
        if self.mode == SignerMode::ReadOnly {
            return signer.map_or(Ok(()), |_| Err(SignerError::UnexpectedSigner));
        }
        let signer = signer.ok_or(SignerError::MissingSigner)?;
        if signer != self.expected_signer {
            return Err(SignerError::WrongSigner);
        }
        Ok(())
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SignerError {
    MissingSigner,
    WrongSigner,
    UnexpectedSigner,
}

impl fmt::Display for SignerError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::MissingSigner => "required signer is unavailable",
            Self::WrongSigner => "provided signer does not match the explicit policy",
            Self::UnexpectedSigner => "read-only operation cannot receive a signer",
        })
    }
}

impl std::error::Error for SignerError {}
