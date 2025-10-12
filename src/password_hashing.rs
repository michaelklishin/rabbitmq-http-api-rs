// Copyright (C) 2023-2025 RabbitMQ Core Team (teamrabbitmq@gmail.com)
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! Functions that calculate salted password hash values the same way RabbitMQ
//! nodes do it.
//!
//! Use this module if you need to calculate a password hash for a `crate::requests::UserParams`
//! object that you pass to [`crate::api::Client::create_user`] or [`crate::blocking_api::Client::create_user`].
//!
//! See the [Credentials and Passwords guide](https://rabbitmq.com/docs/passwords/).
//!

use rand::RngCore;
use ring::digest::{Context, SHA256, SHA512};
use std::fmt;
use thiserror::Error;

const SALT_LENGTH: usize = 4;

/// Generates and returns a 32-bit salt.
/// Used in combination with [`base64_encoded_salted_password_hash_sha256`].
/// See the [Credentials and Passwords guide](https://rabbitmq.com/docs/passwords/).
pub fn salt() -> Vec<u8> {
    // salts are 32 bit long
    let mut buf: [u8; SALT_LENGTH] = [0; SALT_LENGTH];
    let mut rng = rand::rng();
    rng.fill_bytes(&mut buf);
    Vec::from(&buf)
}

/// Produces a SHA-256 hashed, salted password hash.
/// Prefer [`base64_encoded_salted_password_hash_sha256`].
///
/// See the [Credentials and Passwords guide](https://rabbitmq.com/docs/passwords/).
pub fn salted_password_hash_sha256(salt: &[u8], password: &str) -> Vec<u8> {
    salted_password_hash(salt, password, &SHA256)
}

/// Produces a SHA-512 hashed, salted password hash.
/// Prefer [`base64_encoded_salted_password_hash_sha512`].
///
/// See the [Credentials and Passwords guide](https://rabbitmq.com/docs/passwords/).
pub fn salted_password_hash_sha512(salt: &[u8], password: &str) -> Vec<u8> {
    salted_password_hash(salt, password, &SHA512)
}

/// Produces a Base64-encoded, salted password hash using the specified algorithm.
///
/// See the [Credentials and Passwords guide](https://rabbitmq.com/docs/passwords/).
pub fn base64_encoded_salted_password_hash(
    salt: &[u8],
    password: &str,
    algorithm: &HashingAlgorithm,
) -> String {
    let salted = match algorithm {
        HashingAlgorithm::SHA256 => salted_password_hash_sha256(salt, password),
        HashingAlgorithm::SHA512 => salted_password_hash_sha512(salt, password),
    };
    rbase64::encode(salted.as_slice())
}

///
/// Produces a Base64-encoded, SHA-256 hashed, salted password hash that can be passed
/// as [`crate::requests::UserParams::password_hash`] when adding a user with [`crate::blocking_api::Client::create_user`].
///
/// See the [Credentials and Passwords guide](https://rabbitmq.com/docs/passwords/).
pub fn base64_encoded_salted_password_hash_sha256(salt: &[u8], password: &str) -> String {
    base64_encoded_salted_password_hash(salt, password, &HashingAlgorithm::SHA256)
}

///
/// Produces a Base64-encoded, SHA-512 hashed, salted password hash that can be passed
/// as [`crate::requests::UserParams::password_hash`] when adding a user with [`crate::blocking_api::Client::create_user`].
///
/// See the [Credentials and Passwords guide](https://rabbitmq.com/docs/passwords/).
pub fn base64_encoded_salted_password_hash_sha512(salt: &[u8], password: &str) -> String {
    base64_encoded_salted_password_hash(salt, password, &HashingAlgorithm::SHA512)
}

/// Supported hashing algorithms for password salting and hashing.
///
/// This enum represents the cryptographic hash algorithms that can be used
/// for password hashing in RabbitMQ user management. SHA-256 is the default
/// algorithm and is recommended for most use cases.
///
/// See the [Credentials and Passwords guide](https://rabbitmq.com/docs/passwords/).
#[derive(Clone, Default, PartialEq, Eq, Hash, Debug)]
pub enum HashingAlgorithm {
    /// SHA-256 hashing algorithm (default)
    #[default]
    SHA256,
    /// SHA-512 hashing algorithm
    SHA512,
    // Unlike RabbitMQ that accepts module implementations via configuration,
    // we cannot support salting and hashing for arbitrary algorithm names,
    // so Other(String) is omitted by design
}

impl fmt::Display for HashingAlgorithm {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            HashingAlgorithm::SHA256 => write!(f, "SHA-256"),
            HashingAlgorithm::SHA512 => write!(f, "SHA-512"),
        }
    }
}

impl From<&str> for HashingAlgorithm {
    /// Converts a string slice to a `HashingAlgorithm`.
    ///
    /// Supported string values (case-insensitive):
    /// - "SHA256" or "SHA-256" → `HashingAlgorithm::SHA256`
    /// - "SHA512" or "SHA-512" → `HashingAlgorithm::SHA512`
    ///
    /// Any other value defaults to `HashingAlgorithm::SHA256`.
    fn from(s: &str) -> Self {
        match s.to_uppercase().as_str() {
            "SHA256" => HashingAlgorithm::SHA256,
            "SHA-256" => HashingAlgorithm::SHA256,
            "SHA512" => HashingAlgorithm::SHA512,
            "SHA-512" => HashingAlgorithm::SHA512,
            _ => HashingAlgorithm::default(),
        }
    }
}

impl From<String> for HashingAlgorithm {
    fn from(s: String) -> Self {
        HashingAlgorithm::from(s.as_str())
    }
}

/// Errors that can occur during password hashing operations.
#[derive(Error, Debug)]
pub enum HashingError {
    /// The specified hashing algorithm is not supported.
    #[error("Provided algorithm is not supported")]
    UnsupportedAlgorithm,
}

impl HashingAlgorithm {
    /// Generates a Base64-encoded, salted password hash from a clear text password value.
    /// Use this function to produce a password hash to be passed to ``
    ///
    /// This method combines the salt with the password and applies the hash algorithm
    /// specified by this enum variant, then returns the result as a Base64-encoded string.
    ///
    /// # Arguments
    ///
    /// * `salt`: a byte slice containing the salt to use for hashing
    /// * `password`: the plaintext password to hash
    ///
    /// # Returns
    ///
    /// A `Result` containing either:
    /// - `Ok(String)`: the Base64-encoded salted password hash
    /// - `Err(HashingError)`: tn error if the algorithm is not supported
    ///
    /// # Examples
    ///
    /// ```rust
    /// use rabbitmq_http_client::password_hashing::{HashingAlgorithm, salt};
    ///
    /// let salt = salt();
    /// let algorithm = HashingAlgorithm::SHA256;
    /// let hash = algorithm.salt_and_hash(&salt, "a_cleartext_password").unwrap();
    /// ```
    ///
    /// See the [Credentials and Passwords guide](https://rabbitmq.com/docs/passwords/).
    pub fn salt_and_hash(&self, salt: &[u8], password: &str) -> Result<String, HashingError> {
        Ok(base64_encoded_salted_password_hash(salt, password, self))
    }
}

//
// Implementation
//

fn salted_password_hash(
    salt: &[u8],
    password: &str,
    algo: &'static ring::digest::Algorithm,
) -> Vec<u8> {
    let mut ctx = Context::new(algo);
    let vec = [salt, password.as_bytes()].concat();

    ctx.update(&vec);
    let digest = ctx.finish();
    let digest_vec = Vec::from(digest.as_ref());

    [salt, &digest_vec[..]].concat()
}
