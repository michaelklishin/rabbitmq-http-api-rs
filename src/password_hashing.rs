// Copyright (C) 2023-2024 RabbitMQ Core Team (teamrabbitmq@gmail.com)
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
use rand::distributions::{Alphanumeric, DistString};
use ring::digest::{Context, SHA256};

const SALT_LENGTH: usize = 4;

/// Generates and returns a 32-bit salt.
/// Used in combination with [`base64_encoded_salted_password_hash_sha256`].
/// See the [Credentials and Passwords guide](https://rabbitmq.com/docs/passwords/).
pub fn salt() -> Vec<u8> {
    // salts are 32 bit long
    let sample = Alphanumeric.sample_string(&mut rand::thread_rng(), SALT_LENGTH);
    let bytes = sample.as_bytes();
    Vec::from(bytes)
}

/// Produces a SHA-256 hashed, salted passowrd hash.
/// Prefer [`base64_encoded_salted_password_hash_sha256`].
///
/// See the [Credentials and Passwords guide](https://rabbitmq.com/docs/passwords/).
pub fn salted_password_hash_sha256(salt: &[u8], password: &str) -> Vec<u8> {
    let mut ctx = Context::new(&SHA256);
    let vec = [salt, password.as_bytes()].concat();

    ctx.update(&vec);
    let digest = ctx.finish();
    let digest_vec = Vec::from(digest.as_ref());

    [salt, &digest_vec[..]].concat()
}

///
/// Produces a Base64-encoded, SHA-256 hashed, salted passowrd hash that can be passed
/// as [`crate::requests::UserParams::password_hash`] when adding a user with [`crate::blocking::Client::create_user`].
///
/// See the [Credentials and Passwords guide](https://rabbitmq.com/docs/passwords/).
pub fn base64_encoded_salted_password_hash_sha256(salt: &[u8], password: &str) -> String {
    let salted = salted_password_hash_sha256(salt, password);
    rbase64::encode(salted.as_slice())
}
