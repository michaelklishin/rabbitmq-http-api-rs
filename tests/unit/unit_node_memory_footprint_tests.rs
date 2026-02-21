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

use rabbitmq_http_client::responses::cluster::{
    NodeMemoryBreakdown, NodeMemoryFootprint, NodeMemoryTotals,
};
use serde_json::json;

#[test]
fn test_unit_node_memory_footprint_with_breakdown() {
    let json = json!({
        "memory": {
            "connection_readers": 1000,
            "connection_writers": 2000,
            "connection_channels": 3000,
            "connection_other": 4000,
            "queue_procs": 5000,
            "quorum_queue_procs": 6000,
            "stream_queue_procs": 7000,
            "stream_queue_replica_reader_procs": 8000,
            "stream_queue_coordinator_procs": 9000,
            "plugins": 10000,
            "metadata_store": 11000,
            "other_proc": 12000,
            "metrics": 13000,
            "mgmt_db": 14000,
            "mnesia": 15000,
            "quorum_ets": 16000,
            "metadata_store_ets": 17000,
            "other_ets": 18000,
            "binary": 19000,
            "msg_index": 20000,
            "code": 21000,
            "atom": 22000,
            "other_system": 23000,
            "allocated_unused": 24000,
            "reserved_unallocated": 25000,
            "strategy": "allocated",
            "total": {
                "rss": 100000,
                "allocated": 200000,
                "erlang": 150000
            }
        }
    });

    let footprint: NodeMemoryFootprint = serde_json::from_value(json).unwrap();

    assert!(footprint.breakdown.is_some());
    let breakdown = footprint.breakdown.unwrap();

    assert_eq!(breakdown.connection_readers, 1000);
    assert_eq!(breakdown.connection_writers, 2000);
    assert_eq!(breakdown.connection_channels, 3000);
    assert_eq!(breakdown.connection_other, 4000);
    assert_eq!(breakdown.classic_queue_procs, 5000);
    assert_eq!(breakdown.quorum_queue_procs, 6000);
    assert_eq!(breakdown.stream_queue_procs, 7000);
    assert_eq!(breakdown.stream_queue_replica_reader_procs, 8000);
    assert_eq!(breakdown.stream_queue_coordinator_procs, 9000);
    assert_eq!(breakdown.plugins, 10000);
    assert_eq!(breakdown.metadata_store, 11000);
    assert_eq!(breakdown.other_procs, 12000);
    assert_eq!(breakdown.metrics, 13000);
    assert_eq!(breakdown.management_db, 14000);
    assert_eq!(breakdown.mnesia, 15000);
    assert_eq!(breakdown.quorum_queue_ets_tables, 16000);
    assert_eq!(breakdown.metadata_store_ets_tables, 17000);
    assert_eq!(breakdown.other_ets_tables, 18000);
    assert_eq!(breakdown.binary_heap, 19000);
    assert_eq!(breakdown.message_indices, 20000);
    assert_eq!(breakdown.code, 21000);
    assert_eq!(breakdown.atom_table, 22000);
    assert_eq!(breakdown.other_system, 23000);
    assert_eq!(breakdown.allocated_but_unused, 24000);
    assert_eq!(breakdown.reserved_but_unallocated, 25000);
    assert_eq!(breakdown.calculation_strategy, "allocated");

    assert_eq!(breakdown.total.rss, 100000);
    assert_eq!(breakdown.total.allocated, 200000);
    assert_eq!(breakdown.total.used_by_runtime, 150000);
}

#[test]
fn test_unit_node_memory_footprint_not_available() {
    let json = json!({
        "memory": "not_available"
    });

    let footprint: NodeMemoryFootprint = serde_json::from_value(json).unwrap();

    assert!(footprint.breakdown.is_none());
}

#[test]
fn test_unit_node_memory_footprint_invalid_string() {
    let json = json!({
        "memory": "some_other_string"
    });

    let result: Result<NodeMemoryFootprint, _> = serde_json::from_value(json);

    // Should fail to deserialize when the string is not "not_available"
    assert!(result.is_err());
}

#[test]
fn test_unit_node_memory_breakdown_grand_total() {
    let total = NodeMemoryTotals {
        rss: 100000,
        allocated: 200000,
        used_by_runtime: 150000,
    };

    let breakdown = NodeMemoryBreakdown {
        connection_readers: 1000,
        connection_writers: 2000,
        connection_channels: 3000,
        connection_other: 4000,
        classic_queue_procs: 5000,
        quorum_queue_procs: 6000,
        stream_queue_procs: 7000,
        stream_queue_replica_reader_procs: 8000,
        stream_queue_coordinator_procs: 9000,
        plugins: 10000,
        metadata_store: 11000,
        other_procs: 12000,
        metrics: 13000,
        management_db: 14000,
        mnesia: 15000,
        quorum_queue_ets_tables: 16000,
        metadata_store_ets_tables: 17000,
        other_ets_tables: 18000,
        binary_heap: 19000,
        message_indices: 20000,
        code: 21000,
        atom_table: 22000,
        other_system: 23000,
        allocated_but_unused: 24000,
        reserved_but_unallocated: 25000,
        calculation_strategy: "allocated".to_string(),
        total,
    };

    // grand_total should return the maximum of all totals
    assert_eq!(breakdown.grand_total(), 200000);
}

#[test]
fn test_unit_node_memory_totals_max() {
    let totals = NodeMemoryTotals {
        rss: 100000,
        allocated: 200000,
        used_by_runtime: 150000,
    };

    // max should return the maximum value
    assert_eq!(totals.max(), 200000);

    let totals2 = NodeMemoryTotals {
        rss: 300000,
        allocated: 200000,
        used_by_runtime: 150000,
    };

    assert_eq!(totals2.max(), 300000);
}
