//! Error types

use crate::{chain, prost};
use abscissa::Error;
use signatory;
use std::{
    any::Any,
    error::Error as StdError,
    fmt::{self, Display},
    io,
};
use tendermint::amino_types::validate::ValidationError;

/// Error type
#[derive(Debug)]
pub struct KmsError(Error<KmsErrorKind>);

impl KmsError {
    /// Create an error from a panic
    pub fn from_panic(msg: &dyn Any) -> Self {
        if let Some(e) = msg.downcast_ref::<String>() {
            err!(KmsErrorKind::PanicError, e)
        } else if let Some(e) = msg.downcast_ref::<&str>() {
            err!(KmsErrorKind::PanicError, e)
        } else {
            err!(KmsErrorKind::PanicError, "unknown cause")
        }
        .into()
    }
}

/// Kinds of errors
#[derive(Copy, Clone, Eq, PartialEq, Debug, Fail)]
pub enum KmsErrorKind {
    /// Access denied
    #[fail(display = "access denied")]
    #[cfg(feature = "yubihsm")]
    AccessError,

    /// Error in configuration file
    #[fail(display = "config error")]
    ConfigError,

    /// KMS internal panic
    #[fail(display = "internal crash")]
    PanicError,

    /// Cryptographic operation failed
    #[fail(display = "cryptographic error")]
    CryptoError,

    /// Error running a subcommand to update chain state
    #[fail(display = "subcommand hook failed")]
    HookError,

    /// Malformatted or otherwise invalid cryptographic key
    #[fail(display = "invalid key")]
    InvalidKey,

    /// Validation of consensus message failed
    #[fail(display = "invalid consensus message")]
    InvalidMessageError,

    /// Input/output error
    #[fail(display = "I/O error")]
    IoError,

    /// Parse error
    #[fail(display = "parse error")]
    ParseError,

    /// Network protocol-related errors
    #[fail(display = "protocol error")]
    ProtocolError,

    /// Serialization error
    #[fail(display = "serialization error")]
    SerializationError,

    /// Signing operation failed
    #[fail(display = "signing operation failed")]
    SigningError,

    /// Verification operation failed
    #[fail(display = "verification failed")]
    VerificationError,

    /// Signature invalid
    #[fail(display = "attempted double sign")]
    DoubleSign,

    ///Request a Signature above max height
    #[fail(display = "requested signature above stop height")]
    ExceedMaxHeight,
}

impl Display for KmsError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl From<Error<KmsErrorKind>> for KmsError {
    fn from(other: Error<KmsErrorKind>) -> Self {
        KmsError(other)
    }
}

impl From<io::Error> for KmsError {
    fn from(other: io::Error) -> Self {
        err!(KmsErrorKind::IoError, other).into()
    }
}

impl From<prost::DecodeError> for KmsError {
    fn from(other: prost::DecodeError) -> Self {
        err!(KmsErrorKind::ProtocolError, other).into()
    }
}

impl From<prost::EncodeError> for KmsError {
    fn from(other: prost::EncodeError) -> Self {
        err!(KmsErrorKind::ProtocolError, other).into()
    }
}

impl From<serde_json::error::Error> for KmsError {
    fn from(other: serde_json::error::Error) -> Self {
        err!(KmsErrorKind::SerializationError, other).into()
    }
}

impl From<signatory::Error> for KmsError {
    fn from(other: signatory::Error) -> Self {
        let kind = match other.kind() {
            signatory::ErrorKind::Io => KmsErrorKind::IoError,
            signatory::ErrorKind::KeyInvalid => KmsErrorKind::InvalidKey,
            signatory::ErrorKind::ParseError => KmsErrorKind::ParseError,
            signatory::ErrorKind::ProviderError => KmsErrorKind::SigningError,
            signatory::ErrorKind::SignatureInvalid => KmsErrorKind::VerificationError,
        };

        Error::new(kind, Some(other.description().to_owned())).into()
    }
}

impl From<tendermint::Error> for KmsError {
    fn from(other: tendermint::error::Error) -> Self {
        let kind = match other {
            tendermint::Error::Crypto => KmsErrorKind::CryptoError,
            tendermint::Error::InvalidKey => KmsErrorKind::InvalidKey,
            tendermint::Error::Io => KmsErrorKind::IoError,
            tendermint::Error::Protocol => KmsErrorKind::ProtocolError,
            tendermint::Error::Length
            | tendermint::Error::Parse
            | tendermint::Error::OutOfRange => KmsErrorKind::ParseError,
            tendermint::Error::SignatureInvalid => KmsErrorKind::VerificationError,
        };

        Error::new(kind, None).into()
    }
}

impl From<ValidationError> for KmsError {
    fn from(other: ValidationError) -> Self {
        err!(KmsErrorKind::InvalidMessageError, other).into()
    }
}

impl From<chain::state::StateError> for KmsError {
    fn from(other: chain::state::StateError) -> Self {
        err!(KmsErrorKind::DoubleSign, other).into()
    }
}
