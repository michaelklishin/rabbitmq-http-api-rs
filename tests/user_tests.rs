extern crate rabbitmq_http_client;
use rabbitmq_http_client::{blocking::Client, password_hashing, requests::UserParams};

extern crate common;
use crate::common::{endpoint, PASSWORD, USERNAME};

#[test]
fn test_list_users() {
    let endpoint = endpoint();
    let rc = Client::new_with_basic_auth_credentials(&endpoint, USERNAME, PASSWORD);
    let result = rc.list_users();

    assert!(result.is_ok());
    let vec = result.unwrap();
    assert!(vec.iter().find(|u| u.name == "guest").is_some())
}

#[test]
fn test_get_user() {
    let endpoint = endpoint();
    let rc = Client::new_with_basic_auth_credentials(&endpoint, USERNAME, PASSWORD);
    let name = "guest";
    let result = rc.get_user(name);

    assert!(result.is_ok());
    let u = result.unwrap();
    assert!(u.name == name);
}

#[test]
fn test_user_creation() {
    let endpoint = endpoint();
    let rc = Client::new_with_basic_auth_credentials(&endpoint, USERNAME, PASSWORD);

    let salt = password_hashing::salt();
    let password_hash =
        password_hashing::base64_encoded_salted_password_hash_sha256(&salt, &"rust3_t0p_sEkr37");

    let params = UserParams {
        name: "rust3",
        password_hash: &password_hash,
        tags: "management",
    };
    let result = rc.create_user(&params);
    assert!(result.is_ok());
}

#[test]
fn test_user_deletion() {
    let endpoint = endpoint();
    let rc = Client::new_with_basic_auth_credentials(&endpoint, USERNAME, PASSWORD);

    let salt = password_hashing::salt();
    let password_hash =
        password_hashing::base64_encoded_salted_password_hash_sha256(&salt, &"del3te_me");

    let name = "del3te_me";
    let params = UserParams {
        name: name,
        password_hash: &password_hash,
        tags: "management",
    };
    let result1 = rc.create_user(&params);
    assert!(result1.is_ok());

    let result2 = rc.delete_user(&name);
    assert!(result2.is_ok());
}
