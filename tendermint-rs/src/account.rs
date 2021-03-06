//! Tendermint accounts

use crate::error::Error;
#[cfg(feature = "serde")]
use serde::{de, Deserialize, Deserializer, Serialize, Serializer};
use sha2::{Digest, Sha256};
use signatory::ecdsa::curve::secp256k1;
use std::{
    fmt::{self, Debug, Display},
    str::FromStr,
};
use subtle::{self, ConstantTimeEq};
use subtle_encoding::hex;

/// Size of an  account ID in bytes
const LENGTH: usize = 20;

/// Account IDs
#[derive(Copy, Clone, Hash)]
pub struct Id([u8; LENGTH]);

impl Id {
    /// Create a new account ID from raw bytes
    pub fn new(bytes: [u8; LENGTH]) -> Id {
        Id(bytes)
    }

    /// Borrow the account ID as a byte slice
    pub fn as_bytes(&self) -> &[u8] {
        &self.0[..]
    }
}

impl AsRef<[u8]> for Id {
    fn as_ref(&self) -> &[u8] {
        self.as_bytes()
    }
}

impl ConstantTimeEq for Id {
    #[inline]
    fn ct_eq(&self, other: &Id) -> subtle::Choice {
        self.as_bytes().ct_eq(other.as_bytes())
    }
}

impl Display for Id {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for byte in &self.0 {
            write!(f, "{:02X}", byte)?;
        }
        Ok(())
    }
}

impl Debug for Id {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "account::Id({})", self)
    }
}

impl From<secp256k1::PublicKey> for Id {
    fn from(pk: secp256k1::PublicKey) -> Id {
        let digest = Sha256::digest(pk.as_bytes());
        let mut bytes = [0u8; LENGTH];
        bytes.copy_from_slice(&digest[..LENGTH]);
        Id(bytes)
    }
}

/// Decode account ID from hex
impl FromStr for Id {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        // Accept either upper or lower case hex
        let bytes = hex::decode_upper(s)
            .or_else(|_| hex::decode(s))
            .map_err(|_| Error::Parse)?;

        if bytes.len() != LENGTH {
            return Err(Error::Parse);
        }

        let mut result_bytes = [0u8; LENGTH];
        result_bytes.copy_from_slice(&bytes);
        Ok(Id(result_bytes))
    }
}

#[cfg(feature = "serde")]
impl<'de> Deserialize<'de> for Id {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        Self::from_str(&s).map_err(|_| {
            de::Error::custom(format!(
                "expected {}-character hex string, got {:?}",
                LENGTH * 2,
                s
            ))
        })
    }
}

#[cfg(feature = "serde")]
impl Serialize for Id {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        self.to_string().serialize(serializer)
    }
}
