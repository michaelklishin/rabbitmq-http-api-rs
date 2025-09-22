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

use proptest::prelude::*;
use proptest::test_runner::Config as ProptestConfig;
use rabbitmq_http_client::password_hashing::{
    HashingAlgorithm, base64_encoded_salted_password_hash,
    base64_encoded_salted_password_hash_sha256, base64_encoded_salted_password_hash_sha512,
    salted_password_hash_sha256, salted_password_hash_sha512, salt,
};

fn arb_password() -> impl Strategy<Value = String> {
    prop::string::string_regex(r"[a-zA-Z0-9!@#$%^&*()_+-=]{1,64}").unwrap()
}

fn arb_salt() -> impl Strategy<Value = Vec<u8>> {
    prop::collection::vec(any::<u8>(), 4..=4)
}

fn arb_hashing_algorithm() -> impl Strategy<Value = HashingAlgorithm> {
    prop_oneof![
        Just(HashingAlgorithm::SHA256),
        Just(HashingAlgorithm::SHA512),
    ]
}

fn arb_algorithm_string() -> impl Strategy<Value = String> {
    prop_oneof![
        Just("SHA256".to_string()),
        Just("SHA-256".to_string()),
        Just("sha256".to_string()),
        Just("sha-256".to_string()),
        Just("SHA512".to_string()),
        Just("SHA-512".to_string()),
        Just("sha512".to_string()),
        Just("sha-512".to_string()),
        Just("invalid".to_string()),
        Just("".to_string()),
    ]
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(50))]

    #[test]
    fn prop_salt_generates_correct_length(_dummy in Just(1u8)) {
        let result1 = salt();
        prop_assert_eq!(result1.len(), 4);
    }

    #[test]
    fn prop_salt_generates_different_values(_dummy in Just(1u8)) {
        let result1 = salt();
        let result2 = salt();
        prop_assert_ne!(result1, result2);
    }

    #[test]
    fn prop_salted_password_hash_sha256_deterministic(
        salt in arb_salt(),
        password in arb_password()
    ) {
        let result1 = salted_password_hash_sha256(&salt, &password);
        let result2 = salted_password_hash_sha256(&salt, &password);
        prop_assert_eq!(result1, result2);
    }

    #[test]
    fn prop_salted_password_hash_sha512_deterministic(
        salt in arb_salt(),
        password in arb_password()
    ) {
        let result1 = salted_password_hash_sha512(&salt, &password);
        let result2 = salted_password_hash_sha512(&salt, &password);
        prop_assert_eq!(result1, result2);
    }

    #[test]
    fn prop_salted_password_hash_includes_salt(
        salt in arb_salt(),
        password in arb_password()
    ) {
        let result1 = salted_password_hash_sha256(&salt, &password);
        prop_assert!(result1.starts_with(&salt));

        let result2 = salted_password_hash_sha512(&salt, &password);
        prop_assert!(result2.starts_with(&salt));
    }

    #[test]
    fn prop_salted_password_hash_different_salts_different_hashes(
        salt1 in arb_salt(),
        salt2 in arb_salt(),
        password in arb_password()
    ) {
        prop_assume!(salt1 != salt2);

        let result1 = salted_password_hash_sha256(&salt1, &password);
        let result2 = salted_password_hash_sha256(&salt2, &password);
        prop_assert_ne!(result1, result2);
    }

    #[test]
    fn prop_salted_password_hash_different_passwords_different_hashes(
        salt in arb_salt(),
        password1 in arb_password(),
        password2 in arb_password()
    ) {
        prop_assume!(password1 != password2);

        let result1 = salted_password_hash_sha256(&salt, &password1);
        let result2 = salted_password_hash_sha256(&salt, &password2);
        prop_assert_ne!(result1, result2);
    }

    #[test]
    fn prop_sha256_sha512_produce_different_hashes(
        salt in arb_salt(),
        password in arb_password()
    ) {
        let result1 = salted_password_hash_sha256(&salt, &password);
        let result2 = salted_password_hash_sha512(&salt, &password);
        prop_assert_ne!(result1, result2);
    }

    #[test]
    fn prop_base64_encoded_hash_sha256_valid_base64(
        salt in arb_salt(),
        password in arb_password()
    ) {
        let result1 = base64_encoded_salted_password_hash_sha256(&salt, &password);
        prop_assert!(rbase64::decode(&result1).is_ok());
    }

    #[test]
    fn prop_base64_encoded_hash_sha512_valid_base64(
        salt in arb_salt(),
        password in arb_password()
    ) {
        let result1 = base64_encoded_salted_password_hash_sha512(&salt, &password);
        prop_assert!(rbase64::decode(&result1).is_ok());
    }

    #[test]
    fn prop_base64_encoded_hash_deterministic(
        salt in arb_salt(),
        password in arb_password(),
        algorithm in arb_hashing_algorithm()
    ) {
        let result1 = base64_encoded_salted_password_hash(&salt, &password, &algorithm);
        let result2 = base64_encoded_salted_password_hash(&salt, &password, &algorithm);
        prop_assert_eq!(result1, result2);
    }

    #[test]
    fn prop_base64_encoded_hash_matches_raw_hash(
        salt in arb_salt(),
        password in arb_password()
    ) {
        let raw_hash = salted_password_hash_sha256(&salt, &password);
        let base64_hash = base64_encoded_salted_password_hash_sha256(&salt, &password);
        let decoded = rbase64::decode(&base64_hash).unwrap();
        prop_assert_eq!(raw_hash, decoded);
    }

    #[test]
    fn prop_hashing_algorithm_display(algorithm in arb_hashing_algorithm()) {
        let result1 = format!("{}", algorithm);
        match algorithm {
            HashingAlgorithm::SHA256 => prop_assert_eq!(result1, "SHA-256"),
            HashingAlgorithm::SHA512 => prop_assert_eq!(result1, "SHA-512"),
        }
    }

    #[test]
    fn prop_hashing_algorithm_from_string(s in arb_algorithm_string()) {
        let result1 = HashingAlgorithm::from(s.as_str());
        let result2 = HashingAlgorithm::from(s.clone());
        prop_assert_eq!(result1, result2);
    }

    #[test]
    fn prop_hashing_algorithm_from_valid_strings(_dummy in Just(1u8)) {
        let sha256_variants = vec!["SHA256", "SHA-256", "sha256", "sha-256"];
        let sha512_variants = vec!["SHA512", "SHA-512", "sha512", "sha-512"];

        for variant in sha256_variants {
            let result1 = HashingAlgorithm::from(variant);
            prop_assert_eq!(result1, HashingAlgorithm::SHA256);
        }

        for variant in sha512_variants {
            let result1 = HashingAlgorithm::from(variant);
            prop_assert_eq!(result1, HashingAlgorithm::SHA512);
        }
    }

    #[test]
    fn prop_hashing_algorithm_invalid_strings_default_to_sha256(
        invalid_string in prop::string::string_regex(r"[^Ss][^Hh][^Aa][0-9a-zA-Z]{1,10}").unwrap()
    ) {
        prop_assume!(!invalid_string.to_uppercase().contains("SHA"));
        let result1 = HashingAlgorithm::from(invalid_string.as_str());
        prop_assert_eq!(result1, HashingAlgorithm::SHA256);
    }

    #[test]
    fn prop_hashing_algorithm_salt_and_hash_success(
        salt in arb_salt(),
        password in arb_password(),
        algorithm in arb_hashing_algorithm()
    ) {
        let result1 = algorithm.salt_and_hash(&salt, &password);
        prop_assert!(result1.is_ok());

        let hash = result1.unwrap();
        prop_assert!(rbase64::decode(&hash).is_ok());
    }

    #[test]
    fn prop_hashing_algorithm_salt_and_hash_matches_direct_functions(
        salt in arb_salt(),
        password in arb_password()
    ) {
        let sha256_via_enum = HashingAlgorithm::SHA256.salt_and_hash(&salt, &password).unwrap();
        let sha256_direct = base64_encoded_salted_password_hash_sha256(&salt, &password);
        prop_assert_eq!(sha256_via_enum, sha256_direct);

        let sha512_via_enum = HashingAlgorithm::SHA512.salt_and_hash(&salt, &password).unwrap();
        let sha512_direct = base64_encoded_salted_password_hash_sha512(&salt, &password);
        prop_assert_eq!(sha512_via_enum, sha512_direct);
    }

    #[test]
    fn prop_hashing_algorithm_default_is_sha256(_dummy in Just(1u8)) {
        let result1 = HashingAlgorithm::default();
        prop_assert_eq!(result1, HashingAlgorithm::SHA256);
    }

    #[test]
    fn prop_hash_length_consistency_sha256(
        salt1 in arb_salt(),
        salt2 in arb_salt(),
        password1 in arb_password(),
        password2 in arb_password()
    ) {
        let result1 = salted_password_hash_sha256(&salt1, &password1);
        let result2 = salted_password_hash_sha256(&salt2, &password2);
        prop_assert_eq!(result1.len(), result2.len());
        prop_assert_eq!(result1.len(), 4 + 32); // 4 bytes salt + 32 bytes SHA256
    }

    #[test]
    fn prop_hash_length_consistency_sha512(
        salt1 in arb_salt(),
        salt2 in arb_salt(),
        password1 in arb_password(),
        password2 in arb_password()
    ) {
        let result1 = salted_password_hash_sha512(&salt1, &password1);
        let result2 = salted_password_hash_sha512(&salt2, &password2);
        prop_assert_eq!(result1.len(), result2.len());
        prop_assert_eq!(result1.len(), 4 + 64); // 4 bytes salt + 64 bytes SHA512
    }

    #[test]
    fn prop_empty_password_produces_valid_hash(salt in arb_salt()) {
        let result1 = salted_password_hash_sha256(&salt, "");
        prop_assert_eq!(result1.len(), 4 + 32);
        prop_assert!(result1.starts_with(&salt));

        let result2 = base64_encoded_salted_password_hash_sha256(&salt, "");
        prop_assert!(rbase64::decode(&result2).is_ok());
    }

    #[test]
    fn prop_unicode_password_support(
        salt in arb_salt(),
        unicode_chars in prop::string::string_regex(r"[αβγδεζηθικλμνξοπρστυφχψω]{1,20}").unwrap()
    ) {
        let result1 = base64_encoded_salted_password_hash_sha256(&salt, &unicode_chars);
        prop_assert!(rbase64::decode(&result1).is_ok());

        let result2 = base64_encoded_salted_password_hash_sha512(&salt, &unicode_chars);
        prop_assert!(rbase64::decode(&result2).is_ok());
    }
}