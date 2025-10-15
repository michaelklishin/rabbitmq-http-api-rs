use rabbitmq_http_client::commons::{BindingDestinationType, PolicyTarget};
use rabbitmq_http_client::responses::definitions::{
    BindingDefinition, ClusterDefinitionSet, ExchangeDefinition, QueueDefinition, XArguments,
};
use rabbitmq_http_client::responses::policies::{Policy, PolicyDefinition};
use rabbitmq_http_client::responses::{
    Permissions, RuntimeParameter, RuntimeParameterValue, TagList, User, VirtualHost,
};
use serde_json::{Map, json};

fn create_test_user(name: &str, tags: &str) -> User {
    User {
        name: name.to_string(),
        tags: TagList(vec![tags.to_string()]),
        password_hash: format!("$2b$12${}", name),
    }
}

fn create_test_vhost(name: &str) -> VirtualHost {
    VirtualHost {
        name: name.to_string(),
        tags: Some(TagList(vec!["production".to_string()])),
        description: Some(format!("Virtual host {}", name)),
        default_queue_type: Some("quorum".to_string()),
        metadata: None,
    }
}

fn create_test_permissions(user: &str, vhost: &str) -> Permissions {
    Permissions {
        user: user.to_string(),
        vhost: vhost.to_string(),
        configure: ".*".to_string(),
        read: ".*".to_string(),
        write: ".*".to_string(),
    }
}

fn create_test_parameter(name: &str, vhost: &str, component: &str) -> RuntimeParameter {
    let mut value_map = Map::new();
    value_map.insert("key".to_string(), json!("value"));

    RuntimeParameter {
        name: name.to_string(),
        vhost: vhost.to_string(),
        component: component.to_string(),
        value: RuntimeParameterValue(value_map),
    }
}

fn create_test_policy(name: &str, vhost: &str, pattern: &str) -> Policy {
    let mut def_map = Map::new();
    def_map.insert("ha-mode".to_string(), json!("all"));

    Policy {
        name: name.to_string(),
        vhost: vhost.to_string(),
        pattern: pattern.to_string(),
        apply_to: PolicyTarget::Queues,
        priority: 0,
        definition: PolicyDefinition(Some(def_map)),
    }
}

fn create_test_queue(name: &str, vhost: &str) -> QueueDefinition {
    let mut args = Map::new();
    args.insert("x-queue-type".to_string(), json!("quorum"));

    QueueDefinition {
        name: name.to_string(),
        vhost: vhost.to_string(),
        durable: true,
        auto_delete: false,
        arguments: XArguments(args),
    }
}

fn create_test_exchange(name: &str, vhost: &str, exchange_type: &str) -> ExchangeDefinition {
    ExchangeDefinition {
        name: name.to_string(),
        vhost: vhost.to_string(),
        exchange_type: exchange_type.to_string(),
        durable: true,
        auto_delete: false,
        arguments: XArguments(Map::new()),
    }
}

fn create_test_binding(
    vhost: &str,
    source: &str,
    destination: &str,
    routing_key: &str,
) -> BindingDefinition {
    BindingDefinition {
        vhost: vhost.to_string(),
        source: source.to_string(),
        destination: destination.to_string(),
        destination_type: BindingDestinationType::Queue,
        routing_key: routing_key.to_string(),
        arguments: XArguments(Map::new()),
        properties_key: None,
    }
}

#[test]
fn test_empty_diff() {
    let def1 = ClusterDefinitionSet {
        server_version: Some("3.13.0".to_string()),
        users: vec![],
        virtual_hosts: vec![],
        permissions: vec![],
        parameters: vec![],
        policies: vec![],
        queues: vec![],
        exchanges: vec![],
        bindings: vec![],
    };

    let def2 = def1.clone();
    let diff = def1.diff(&def2);

    assert!(diff.is_empty());
    assert!(!diff.has_changes());
}

#[test]
fn test_user_added() {
    let def1 = ClusterDefinitionSet {
        server_version: Some("3.13.0".to_string()),
        users: vec![create_test_user("user1", "administrator")],
        virtual_hosts: vec![],
        permissions: vec![],
        parameters: vec![],
        policies: vec![],
        queues: vec![],
        exchanges: vec![],
        bindings: vec![],
    };

    let def2 = ClusterDefinitionSet {
        server_version: Some("3.13.0".to_string()),
        users: vec![
            create_test_user("user1", "administrator"),
            create_test_user("user2", "monitoring"),
        ],
        virtual_hosts: vec![],
        permissions: vec![],
        parameters: vec![],
        policies: vec![],
        queues: vec![],
        exchanges: vec![],
        bindings: vec![],
    };

    let diff = def1.diff(&def2);

    assert!(!diff.is_empty());
    assert!(diff.has_changes());
    assert_eq!(diff.users.only_in_left.len(), 0);
    assert_eq!(diff.users.only_in_right.len(), 1);
    assert_eq!(diff.users.modified.len(), 0);
    assert_eq!(diff.users.only_in_right[0].name, "user2");
}

#[test]
fn test_user_deleted() {
    let def1 = ClusterDefinitionSet {
        server_version: Some("3.13.0".to_string()),
        users: vec![
            create_test_user("user1", "administrator"),
            create_test_user("user2", "monitoring"),
        ],
        virtual_hosts: vec![],
        permissions: vec![],
        parameters: vec![],
        policies: vec![],
        queues: vec![],
        exchanges: vec![],
        bindings: vec![],
    };

    let def2 = ClusterDefinitionSet {
        server_version: Some("3.13.0".to_string()),
        users: vec![create_test_user("user1", "administrator")],
        virtual_hosts: vec![],
        permissions: vec![],
        parameters: vec![],
        policies: vec![],
        queues: vec![],
        exchanges: vec![],
        bindings: vec![],
    };

    let diff = def1.diff(&def2);

    assert!(!diff.is_empty());
    assert_eq!(diff.users.only_in_left.len(), 1);
    assert_eq!(diff.users.only_in_right.len(), 0);
    assert_eq!(diff.users.modified.len(), 0);
    assert_eq!(diff.users.only_in_left[0].name, "user2");
}

#[test]
fn test_user_modified() {
    let def1 = ClusterDefinitionSet {
        server_version: Some("3.13.0".to_string()),
        users: vec![create_test_user("user1", "administrator")],
        virtual_hosts: vec![],
        permissions: vec![],
        parameters: vec![],
        policies: vec![],
        queues: vec![],
        exchanges: vec![],
        bindings: vec![],
    };

    let def2 = ClusterDefinitionSet {
        server_version: Some("3.13.0".to_string()),
        users: vec![create_test_user("user1", "monitoring")],
        virtual_hosts: vec![],
        permissions: vec![],
        parameters: vec![],
        policies: vec![],
        queues: vec![],
        exchanges: vec![],
        bindings: vec![],
    };

    let diff = def1.diff(&def2);

    assert!(!diff.is_empty());
    assert_eq!(diff.users.only_in_left.len(), 0);
    assert_eq!(diff.users.only_in_right.len(), 0);
    assert_eq!(diff.users.modified.len(), 1);
    assert_eq!(
        diff.users.modified[0].0.tags.0,
        vec!["administrator".to_string()]
    );
    assert_eq!(
        diff.users.modified[0].1.tags.0,
        vec!["monitoring".to_string()]
    );
}

#[test]
fn test_vhost_changes() {
    let def1 = ClusterDefinitionSet {
        server_version: Some("3.13.0".to_string()),
        users: vec![],
        virtual_hosts: vec![create_test_vhost("/")],
        permissions: vec![],
        parameters: vec![],
        policies: vec![],
        queues: vec![],
        exchanges: vec![],
        bindings: vec![],
    };

    let def2 = ClusterDefinitionSet {
        server_version: Some("3.13.0".to_string()),
        users: vec![],
        virtual_hosts: vec![create_test_vhost("/"), create_test_vhost("/app")],
        permissions: vec![],
        parameters: vec![],
        policies: vec![],
        queues: vec![],
        exchanges: vec![],
        bindings: vec![],
    };

    let diff = def1.diff(&def2);

    assert!(!diff.is_empty());
    assert_eq!(diff.virtual_hosts.only_in_right.len(), 1);
    assert_eq!(diff.virtual_hosts.only_in_right[0].name, "/app");
}

#[test]
fn test_permissions_by_composite_key() {
    let def1 = ClusterDefinitionSet {
        server_version: Some("3.13.0".to_string()),
        users: vec![],
        virtual_hosts: vec![],
        permissions: vec![
            create_test_permissions("user1", "/"),
            create_test_permissions("user1", "/app"),
        ],
        parameters: vec![],
        policies: vec![],
        queues: vec![],
        exchanges: vec![],
        bindings: vec![],
    };

    let def2 = ClusterDefinitionSet {
        server_version: Some("3.13.0".to_string()),
        users: vec![],
        virtual_hosts: vec![],
        permissions: vec![
            create_test_permissions("user1", "/"),
            create_test_permissions("user2", "/"),
        ],
        parameters: vec![],
        policies: vec![],
        queues: vec![],
        exchanges: vec![],
        bindings: vec![],
    };

    let diff = def1.diff(&def2);

    assert!(!diff.is_empty());
    assert_eq!(diff.permissions.only_in_left.len(), 1);
    assert_eq!(diff.permissions.only_in_right.len(), 1);
    assert_eq!(diff.permissions.only_in_left[0].user, "user1");
    assert_eq!(diff.permissions.only_in_left[0].vhost, "/app");
    assert_eq!(diff.permissions.only_in_right[0].user, "user2");
    assert_eq!(diff.permissions.only_in_right[0].vhost, "/");
}

#[test]
fn test_queue_modifications() {
    let queue1 = create_test_queue("orders", "/");
    let mut queue2 = create_test_queue("orders", "/");
    queue2
        .arguments
        .insert("x-max-length".to_string(), json!(1000));

    let def1 = ClusterDefinitionSet {
        server_version: Some("3.13.0".to_string()),
        users: vec![],
        virtual_hosts: vec![],
        permissions: vec![],
        parameters: vec![],
        policies: vec![],
        queues: vec![queue1],
        exchanges: vec![],
        bindings: vec![],
    };

    let def2 = ClusterDefinitionSet {
        server_version: Some("3.13.0".to_string()),
        users: vec![],
        virtual_hosts: vec![],
        permissions: vec![],
        parameters: vec![],
        policies: vec![],
        queues: vec![queue2],
        exchanges: vec![],
        bindings: vec![],
    };

    let diff = def1.diff(&def2);

    assert!(!diff.is_empty());
    assert_eq!(diff.queues.modified.len(), 1);
    assert_eq!(diff.queues.modified[0].0.name, "orders");
    assert!(
        !diff.queues.modified[0]
            .0
            .arguments
            .contains_key("x-max-length")
    );
    assert!(
        diff.queues.modified[0]
            .1
            .arguments
            .contains_key("x-max-length")
    );
}

#[test]
fn test_policy_changes() {
    let def1 = ClusterDefinitionSet {
        server_version: Some("3.13.0".to_string()),
        users: vec![],
        virtual_hosts: vec![],
        permissions: vec![],
        parameters: vec![],
        policies: vec![create_test_policy("ha-all", "/", ".*")],
        queues: vec![],
        exchanges: vec![],
        bindings: vec![],
    };

    let def2 = ClusterDefinitionSet {
        server_version: Some("3.13.0".to_string()),
        users: vec![],
        virtual_hosts: vec![],
        permissions: vec![],
        parameters: vec![],
        policies: vec![],
        queues: vec![],
        exchanges: vec![],
        bindings: vec![],
    };

    let diff = def1.diff(&def2);

    assert!(!diff.is_empty());
    assert_eq!(diff.policies.only_in_left.len(), 1);
    assert_eq!(diff.policies.only_in_left[0].name, "ha-all");
}

#[test]
fn test_exchange_and_binding_changes() {
    let def1 = ClusterDefinitionSet {
        server_version: Some("3.13.0".to_string()),
        users: vec![],
        virtual_hosts: vec![],
        permissions: vec![],
        parameters: vec![],
        policies: vec![],
        queues: vec![],
        exchanges: vec![create_test_exchange("events", "/", "topic")],
        bindings: vec![create_test_binding("/", "events", "orders", "order.#")],
    };

    let def2 = ClusterDefinitionSet {
        server_version: Some("3.13.0".to_string()),
        users: vec![],
        virtual_hosts: vec![],
        permissions: vec![],
        parameters: vec![],
        policies: vec![],
        queues: vec![],
        exchanges: vec![
            create_test_exchange("events", "/", "topic"),
            create_test_exchange("logs", "/", "topic"),
        ],
        bindings: vec![
            create_test_binding("/", "events", "orders", "order.#"),
            create_test_binding("/", "logs", "app", "*.error"),
        ],
    };

    let diff = def1.diff(&def2);

    assert!(!diff.is_empty());
    assert_eq!(diff.exchanges.only_in_right.len(), 1);
    assert_eq!(diff.exchanges.only_in_right[0].name, "logs");
    assert_eq!(diff.bindings.only_in_right.len(), 1);
    assert_eq!(diff.bindings.only_in_right[0].source, "logs");
}

#[test]
fn test_complex_multiresource_diff() {
    let def1 = ClusterDefinitionSet {
        server_version: Some("3.13.0".to_string()),
        users: vec![
            create_test_user("admin", "administrator"),
            create_test_user("guest", "administrator"),
        ],
        virtual_hosts: vec![create_test_vhost("/")],
        permissions: vec![create_test_permissions("admin", "/")],
        parameters: vec![create_test_parameter("shovel1", "/", "shovel")],
        policies: vec![create_test_policy("ha", "/", ".*")],
        queues: vec![create_test_queue("orders", "/")],
        exchanges: vec![create_test_exchange("amq.topic", "/", "topic")],
        bindings: vec![create_test_binding("/", "amq.topic", "orders", "#")],
    };

    let def2 = ClusterDefinitionSet {
        server_version: Some("3.13.0".to_string()),
        users: vec![
            create_test_user("admin", "management"),
            create_test_user("app_user", "monitoring"),
        ],
        virtual_hosts: vec![create_test_vhost("/"), create_test_vhost("/app")],
        permissions: vec![
            create_test_permissions("admin", "/"),
            create_test_permissions("app_user", "/app"),
        ],
        parameters: vec![],
        policies: vec![create_test_policy("ha", "/", "^prod.*")],
        queues: vec![
            create_test_queue("orders", "/"),
            create_test_queue("events", "/app"),
        ],
        exchanges: vec![create_test_exchange("amq.topic", "/", "topic")],
        bindings: vec![],
    };

    let diff = def1.diff(&def2);

    assert!(diff.has_changes());

    assert_eq!(diff.users.only_in_left.len(), 1);
    assert_eq!(diff.users.only_in_right.len(), 1);
    assert_eq!(diff.users.modified.len(), 1);

    assert_eq!(diff.virtual_hosts.only_in_right.len(), 1);

    assert_eq!(diff.permissions.only_in_right.len(), 1);

    assert_eq!(diff.parameters.only_in_left.len(), 1);

    assert_eq!(diff.policies.modified.len(), 1);

    assert_eq!(diff.queues.only_in_right.len(), 1);

    assert_eq!(diff.bindings.only_in_left.len(), 1);
}

#[test]
fn test_parameter_identity_by_three_fields() {
    let def1 = ClusterDefinitionSet {
        server_version: Some("3.13.0".to_string()),
        users: vec![],
        virtual_hosts: vec![],
        permissions: vec![],
        parameters: vec![
            create_test_parameter("param1", "/", "shovel"),
            create_test_parameter("param1", "/", "federation"),
            create_test_parameter("param1", "/app", "shovel"),
        ],
        policies: vec![],
        queues: vec![],
        exchanges: vec![],
        bindings: vec![],
    };

    let def2 = ClusterDefinitionSet {
        server_version: Some("3.13.0".to_string()),
        users: vec![],
        virtual_hosts: vec![],
        permissions: vec![],
        parameters: vec![
            create_test_parameter("param1", "/", "shovel"),
            create_test_parameter("param1", "/app", "shovel"),
        ],
        policies: vec![],
        queues: vec![],
        exchanges: vec![],
        bindings: vec![],
    };

    let diff = def1.diff(&def2);

    assert!(!diff.is_empty());
    assert_eq!(diff.parameters.only_in_left.len(), 1);
    assert_eq!(diff.parameters.only_in_left[0].component, "federation");
}

#[test]
fn test_binding_identity_with_properties_key() {
    let mut binding1 = create_test_binding("/", "source", "dest", "key");
    binding1.properties_key = Some("props1".to_string());

    let mut binding2 = create_test_binding("/", "source", "dest", "key");
    binding2.properties_key = Some("props2".to_string());

    let def1 = ClusterDefinitionSet {
        server_version: Some("3.13.0".to_string()),
        users: vec![],
        virtual_hosts: vec![],
        permissions: vec![],
        parameters: vec![],
        policies: vec![],
        queues: vec![],
        exchanges: vec![],
        bindings: vec![binding1],
    };

    let def2 = ClusterDefinitionSet {
        server_version: Some("3.13.0".to_string()),
        users: vec![],
        virtual_hosts: vec![],
        permissions: vec![],
        parameters: vec![],
        policies: vec![],
        queues: vec![],
        exchanges: vec![],
        bindings: vec![binding2],
    };

    let diff = def1.diff(&def2);

    assert!(!diff.is_empty());
    assert_eq!(diff.bindings.only_in_left.len(), 1);
    assert_eq!(diff.bindings.only_in_right.len(), 1);
}
