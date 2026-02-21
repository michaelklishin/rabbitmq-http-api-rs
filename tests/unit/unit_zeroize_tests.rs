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

use rabbitmq_http_client::commons::Password;
use rabbitmq_http_client::requests::users::OwnedUserParams;
use rabbitmq_http_client::requests::users::UserParams;
use rabbitmq_http_client::responses::TagList;
use rabbitmq_http_client::responses::users::User;
use zeroize::Zeroize;

#[test]
fn test_owned_user_params_zeroize_clears_all_fields() {
    let mut params = OwnedUserParams::new("admin", "s3cret_hash_value!", "administrator");
    params.zeroize();

    assert!(params.name.is_empty());
    assert!(params.password_hash.is_empty());
    assert!(params.tags.is_empty());
}

#[test]
fn test_owned_user_params_zeroize_with_administrator() {
    let mut params = OwnedUserParams::administrator("admin", "hash_abc123");
    params.zeroize();

    assert!(params.name.is_empty());
    assert!(params.password_hash.is_empty());
    assert!(params.tags.is_empty());
}

#[test]
fn test_owned_user_params_zeroize_with_monitoring() {
    let mut params = OwnedUserParams::monitoring("monitor", "hash_xyz");
    params.zeroize();

    assert!(params.name.is_empty());
    assert!(params.password_hash.is_empty());
    assert!(params.tags.is_empty());
}

#[test]
fn test_owned_user_params_zeroize_with_management() {
    let mut params = OwnedUserParams::management("mgmt", "hash_mgmt");
    params.zeroize();

    assert!(params.name.is_empty());
    assert!(params.password_hash.is_empty());
    assert!(params.tags.is_empty());
}

#[test]
fn test_owned_user_params_zeroize_with_policymaker() {
    let mut params = OwnedUserParams::policymaker("policy", "hash_policy");
    params.zeroize();

    assert!(params.name.is_empty());
    assert!(params.password_hash.is_empty());
    assert!(params.tags.is_empty());
}

#[test]
fn test_owned_user_params_zeroize_without_tags() {
    let mut params = OwnedUserParams::without_tags("basic", "hash_basic");
    params.zeroize();

    assert!(params.name.is_empty());
    assert!(params.password_hash.is_empty());
    assert!(params.tags.is_empty());
}

#[test]
fn test_owned_user_params_zeroize_with_empty_values() {
    let mut params = OwnedUserParams::new("", "", "");
    params.zeroize();

    assert!(params.name.is_empty());
    assert!(params.password_hash.is_empty());
    assert!(params.tags.is_empty());
}

#[test]
fn test_user_response_zeroize_clears_all_fields() {
    let mut user = User {
        name: "admin".to_owned(),
        tags: TagList(vec!["administrator".to_owned()]),
        password_hash: "salted_hash_value".to_owned(),
    };
    user.zeroize();

    assert!(user.name.is_empty());
    assert!(user.password_hash.is_empty());
    assert!(user.tags.is_empty());
}

#[test]
fn test_user_response_zeroize_with_multiple_tags() {
    let mut user = User {
        name: "ops".to_owned(),
        tags: TagList(vec![
            "administrator".to_owned(),
            "monitoring".to_owned(),
            "management".to_owned(),
        ]),
        password_hash: "complex_hash".to_owned(),
    };
    user.zeroize();

    assert!(user.name.is_empty());
    assert!(user.password_hash.is_empty());
    assert!(user.tags.is_empty());
}

#[test]
fn test_user_response_zeroize_with_empty_tags() {
    let mut user = User {
        name: "notags".to_owned(),
        tags: TagList(vec![]),
        password_hash: "some_hash".to_owned(),
    };
    user.zeroize();

    assert!(user.name.is_empty());
    assert!(user.password_hash.is_empty());
    assert!(user.tags.is_empty());
}

#[test]
fn test_tag_list_zeroize() {
    let mut tags = TagList(vec!["administrator".to_owned(), "monitoring".to_owned()]);
    tags.zeroize();

    assert!(tags.is_empty());
}

#[test]
fn test_tag_list_zeroize_empty() {
    let mut tags = TagList(vec![]);
    tags.zeroize();

    assert!(tags.is_empty());
}

#[test]
fn test_zeroize_is_idempotent_on_owned_user_params() {
    let mut params = OwnedUserParams::new("user", "hash", "tags");
    params.zeroize();
    params.zeroize();

    assert!(params.name.is_empty());
    assert!(params.password_hash.is_empty());
    assert!(params.tags.is_empty());
}

#[test]
fn test_zeroize_is_idempotent_on_user() {
    let mut user = User {
        name: "user".to_owned(),
        tags: TagList(vec!["admin".to_owned()]),
        password_hash: "hash".to_owned(),
    };
    user.zeroize();
    user.zeroize();

    assert!(user.name.is_empty());
    assert!(user.password_hash.is_empty());
    assert!(user.tags.is_empty());
}

#[test]
fn test_owned_user_params_clone_then_zeroize_original() {
    let mut original = OwnedUserParams::new("user", "secret_hash", "administrator");
    let cloned = original.clone();

    original.zeroize();

    assert!(original.name.is_empty());
    assert!(original.password_hash.is_empty());
    assert_eq!(cloned.name, "user");
    assert_eq!(cloned.password_hash, "secret_hash");
}

#[test]
fn test_user_clone_then_zeroize_original() {
    let mut original = User {
        name: "user".to_owned(),
        tags: TagList(vec!["admin".to_owned()]),
        password_hash: "secret_hash".to_owned(),
    };
    let cloned = original.clone();

    original.zeroize();

    assert!(original.name.is_empty());
    assert!(original.password_hash.is_empty());
    assert_eq!(cloned.name, "user");
    assert_eq!(cloned.password_hash, "secret_hash");
}

#[test]
fn test_owned_user_params_as_ref_before_zeroize() {
    let params = OwnedUserParams::new("user", "hash", "admin");
    let borrowed = params.as_ref();

    assert_eq!(borrowed.name, "user");
    assert_eq!(borrowed.password_hash, "hash");
    assert_eq!(borrowed.tags, "admin");
}

#[test]
fn test_owned_user_params_from_user_params_then_zeroize() {
    let borrowed = UserParams::administrator("admin", "hash_abc");
    let mut owned = OwnedUserParams::from(borrowed);

    assert_eq!(owned.name, "admin");
    assert_eq!(owned.password_hash, "hash_abc");
    assert_eq!(owned.tags, "administrator");

    owned.zeroize();

    assert!(owned.name.is_empty());
    assert!(owned.password_hash.is_empty());
    assert!(owned.tags.is_empty());
}

#[test]
fn test_owned_user_params_zeroize_with_unicode() {
    let mut params = OwnedUserParams::new("пользователь", "хэш_пароля_密码", "администратор");
    params.zeroize();

    assert!(params.name.is_empty());
    assert!(params.password_hash.is_empty());
    assert!(params.tags.is_empty());
}

#[test]
fn test_user_zeroize_with_unicode() {
    let mut user = User {
        name: "管理者".to_owned(),
        tags: TagList(vec!["管理員".to_owned()]),
        password_hash: "密码哈希".to_owned(),
    };
    user.zeroize();

    assert!(user.name.is_empty());
    assert!(user.password_hash.is_empty());
    assert!(user.tags.is_empty());
}

#[test]
fn test_password_display_is_transparent() {
    let pw = Password::from("s3cret_value");
    assert_eq!(format!("{}", pw), "s3cret_value");
}

#[test]
fn test_password_from_string() {
    let pw = Password::from("hello".to_owned());
    assert_eq!(format!("{}", pw), "hello");
}

#[test]
fn test_password_from_str() {
    let pw = Password::from("world");
    assert_eq!(format!("{}", pw), "world");
}

#[test]
fn test_password_new() {
    let pw = Password::new("secret");
    assert_eq!(format!("{}", pw), "secret");
}

#[test]
fn test_password_zeroize() {
    let mut pw = Password::new("s3cret");
    pw.zeroize();
    assert_eq!(format!("{}", pw), "");
}

#[test]
fn test_password_clone_then_zeroize() {
    let mut original = Password::new("s3cret");
    let cloned = original.clone();

    original.zeroize();

    assert_eq!(format!("{}", original), "");
    assert_eq!(format!("{}", cloned), "s3cret");
}

#[test]
fn test_password_zeroize_empty() {
    let mut pw = Password::new("");
    pw.zeroize();
    assert_eq!(format!("{}", pw), "");
}

#[test]
fn test_password_zeroize_is_idempotent() {
    let mut pw = Password::new("s3cret");
    pw.zeroize();
    pw.zeroize();
    assert_eq!(format!("{}", pw), "");
}

#[test]
fn test_password_zeroize_with_unicode() {
    let mut pw = Password::new("пароль_contraseña_パスワード");
    pw.zeroize();
    assert_eq!(format!("{}", pw), "");
}

#[test]
fn test_password_debug_is_redacted() {
    let pw = Password::new("super_secret_value");
    let debug_output = format!("{:?}", pw);
    assert_eq!(debug_output, "Password([REDACTED])");
    assert!(!debug_output.contains("super_secret_value"));
}
