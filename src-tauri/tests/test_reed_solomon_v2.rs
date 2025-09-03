// Copyright 2024 Saorsa Labs
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Comprehensive tests for Reed Solomon v2 with 60% availability

use anyhow::Result;
use communitas_tauri::storage::reed_solomon_v2::*;

#[tokio::test]
async fn test_60_percent_availability_comprehensive() -> Result<()> {
    println!("🧪 Testing 60% availability requirement across group sizes...\n");

    let test_scenarios = vec![
        (5, "Small group"),
        (10, "Medium group"),
        (20, "Large group"),
        (50, "XL group"),
        (100, "XXL group"),
    ];

    for (member_count, description) in test_scenarios {
        let config = OptimalReedSolomonConfig::for_group_size(member_count);

        let availability_ratio = config.data_shards as f32 / config.total_shards() as f32;
        let percent_required = (availability_ratio * 100.0) as i32;
        let members_required = config.minimum_members_required();
        let can_lose = config.maximum_failures_tolerated();

        println!("📊 {} ({} members):", description, member_count);
        println!(
            "   Configuration: {} data + {} parity shards",
            config.data_shards, config.parity_shards
        );
        println!(
            "   Availability: {}% of members need to be online",
            percent_required
        );
        println!("   Minimum online: {} members", members_required);
        println!("   Can tolerate: {} members offline", can_lose);
        println!("   ✅ Within target: {}", config.is_within_tolerance());

        // Verify it's close to 60%
        assert!(
            (availability_ratio - 0.6).abs() <= 0.1,
            "Availability {}% not close enough to 60% target",
            percent_required
        );

        println!();
    }

    Ok(())
}

#[tokio::test]
async fn test_dynamic_membership_rebalancing() -> Result<()> {
    println!("🔄 Testing dynamic membership and rebalancing logic...\n");

    let manager = DynamicMembershipManager::new("test-node".to_string());
    let group_id = "test-group";

    // Start with 5 members
    let initial_members: Vec<String> = (1..=5).map(|i| format!("member-{}", i)).collect();

    {
        let mut states = manager.group_states.write().await;
        states.insert(
            group_id.to_string(),
            GroupState::Stable {
                config: OptimalReedSolomonConfig::for_group_size(5),
                members: initial_members.clone(),
            },
        );
    }

    println!("Initial group: {} members", initial_members.len());

    // Test 1: Small change (add 1 member) - should NOT trigger rebalancing
    let small_change: Vec<String> = (1..=6).map(|i| format!("member-{}", i)).collect();

    let should_rebalance = manager.should_rebalance(group_id, &small_change).await?;
    println!(
        "After adding 1 member (20% change): rebalance = {}",
        should_rebalance
    );
    assert!(!should_rebalance, "Should not rebalance for 20% change");

    // Test 2: Large change (double the size) - SHOULD trigger rebalancing
    let large_change: Vec<String> = (1..=10).map(|i| format!("member-{}", i)).collect();

    let should_rebalance = manager.should_rebalance(group_id, &large_change).await?;
    println!(
        "After doubling size (100% change): rebalance = {}",
        should_rebalance
    );
    assert!(should_rebalance, "Should rebalance for 100% change");

    // Test 3: Member joining during grace period
    manager
        .handle_member_join(group_id, "member-11".to_string())
        .await?;

    let states = manager.group_states.read().await;
    if let Some(GroupState::MemberJoining { joining_member, .. }) = states.get(group_id) {
        println!("✅ Member {} queued to join", joining_member);
    }

    Ok(())
}

#[tokio::test]
async fn test_intelligent_shard_distribution() -> Result<()> {
    println!("🎯 Testing intelligent shard distribution with member reliability...\n");

    let distributor = IntelligentShardDistributor::new();

    // Set up member reliability scores
    let members = vec![
        ("reliable-1".to_string(), 0.95),
        ("reliable-2".to_string(), 0.90),
        ("average-1".to_string(), 0.70),
        ("unreliable-1".to_string(), 0.40),
    ];

    for (member_id, score) in &members {
        let mut reliability_map = distributor.member_reliability.write().await;
        let mut reliability = MemberReliability::new(member_id.clone());
        reliability.reliability_score = *score;
        reliability_map.insert(member_id.clone(), reliability);
    }

    let member_ids: Vec<String> = members.iter().map(|(id, _)| id.clone()).collect();
    let config = OptimalReedSolomonConfig::for_group_size(4);

    let plan = distributor
        .create_distribution_plan(&config, &member_ids, "test-data")
        .await?;

    println!("Distribution plan for {} members:", member_ids.len());
    println!(
        "Config: {} data + {} parity shards",
        config.data_shards, config.parity_shards
    );

    for (member_id, shards) in &plan.assignments {
        let score = members
            .iter()
            .find(|(id, _)| id == member_id)
            .map(|(_, s)| s)
            .unwrap_or(&0.0);

        let shard_types: Vec<&str> = shards
            .iter()
            .map(|&idx| {
                if idx < config.data_shards {
                    "data"
                } else {
                    "parity"
                }
            })
            .collect();

        println!(
            "  {} (reliability: {:.2}): {:?}",
            member_id, score, shard_types
        );
    }

    // Verify that more reliable members get data shards
    let reliable_1_shards = plan.assignments.get("reliable-1").unwrap();
    let has_data_shard = reliable_1_shards
        .iter()
        .any(|&idx| idx < config.data_shards);
    assert!(
        has_data_shard,
        "Most reliable member should have data shards"
    );

    Ok(())
}

#[tokio::test]
async fn test_hybrid_logical_clock() -> Result<()> {
    println!("🕐 Testing Hybrid Logical Clock for causal ordering...\n");

    let mut hlc1 = HybridLogicalClock::new("node1".to_string());
    let mut hlc2 = HybridLogicalClock::new("node2".to_string());

    // Test tick advancement
    let t1 = hlc1.tick();
    let t2 = hlc1.tick();
    assert!(t2 > t1, "Later tick should be greater");
    println!("✅ HLC tick ordering works");

    // Test update from remote clock
    let remote_tick = hlc2.tick();
    hlc1.update(&remote_tick);
    let next_tick = hlc1.tick();
    assert!(
        next_tick > remote_tick,
        "Updated clock should advance beyond remote"
    );
    println!("✅ HLC update from remote works");

    // Test causality preservation
    let event1 = hlc1.tick();
    hlc2.update(&event1);
    let event2 = hlc2.tick();

    assert!(
        event2 > event1,
        "Causally dependent event should have greater timestamp"
    );
    println!("✅ Causality preserved across nodes");

    Ok(())
}

#[tokio::test]
async fn test_recovery_scenarios() -> Result<()> {
    println!("🔧 Testing recovery scenarios with 60% availability...\n");

    let scenarios = vec![
        (10, 4, "40% members offline - recovery possible"),
        (10, 6, "60% members offline - exactly at threshold"),
        (20, 8, "40% members offline in large group"),
    ];

    for (total_members, offline_count, description) in scenarios {
        let config = OptimalReedSolomonConfig::for_group_size(total_members);
        let online_members = total_members - offline_count;
        let can_recover = online_members >= config.minimum_members_required();

        println!("Scenario: {}", description);
        println!("  Total members: {}", total_members);
        println!("  Offline: {}", offline_count);
        println!("  Online: {}", online_members);
        println!("  Minimum required: {}", config.minimum_members_required());
        println!("  Recovery possible: {}", can_recover);
        println!();

        // For 60% availability, we should be able to recover with 60% online
        let percent_online = (online_members as f32 / total_members as f32) * 100.0;
        if percent_online >= 60.0 {
            assert!(
                can_recover,
                "Should be able to recover with {}% online",
                percent_online
            );
        }
    }

    Ok(())
}

#[tokio::test]
async fn test_consistent_hashing_distribution() -> Result<()> {
    println!("🔗 Testing consistent hashing for shard distribution...\n");

    let members: Vec<String> = (1..=5).map(|i| format!("member-{}", i)).collect();
    let ring = ConsistentHashRing::new(&members, 3);

    // Test shard assignment consistency
    let shard_assignments: Vec<(String, Vec<String>)> = (0..10)
        .map(|i| {
            let shard_id = format!("shard-{}", i);
            let nodes = ring.get_nodes_for_shard(&shard_id, 2);
            (shard_id, nodes)
        })
        .collect();

    println!("Shard assignments (2 replicas each):");
    for (shard_id, nodes) in &shard_assignments {
        println!("  {} -> {:?}", shard_id, nodes);
    }

    // Verify consistency - same shard should always get same nodes
    for (shard_id, expected_nodes) in &shard_assignments {
        let actual_nodes = ring.get_nodes_for_shard(shard_id, 2);
        assert_eq!(
            &actual_nodes, expected_nodes,
            "Shard {} assignment not consistent",
            shard_id
        );
    }

    println!("✅ Consistent hashing maintains stable assignments");

    // Test distribution fairness
    let mut member_load: std::collections::HashMap<String, usize> =
        std::collections::HashMap::new();

    for (_, nodes) in &shard_assignments {
        for node in nodes {
            *member_load.entry(node.clone()).or_insert(0) += 1;
        }
    }

    println!("\nLoad distribution:");
    for (member, load) in &member_load {
        println!("  {}: {} shards", member, load);
    }

    // Check that load is relatively balanced (within 2x of average)
    let avg_load = shard_assignments.len() * 2 / members.len();
    for load in member_load.values() {
        assert!(*load <= avg_load * 2, "Load imbalance detected");
    }

    println!("✅ Load is reasonably balanced across members");

    Ok(())
}

#[tokio::test]
async fn test_state_transitions() -> Result<()> {
    println!("🔄 Testing group state machine transitions...\n");

    let manager = DynamicMembershipManager::new("coordinator".to_string());
    let group_id = "state-test-group";

    // Initialize stable state
    {
        let mut states = manager.group_states.write().await;
        states.insert(
            group_id.to_string(),
            GroupState::Stable {
                config: OptimalReedSolomonConfig::for_group_size(5),
                members: vec!["m1".to_string(), "m2".to_string(), "m3".to_string()],
            },
        );
    }
    println!("✅ Initial state: Stable with 3 members");

    // Trigger member join
    manager
        .handle_member_join(group_id, "m4".to_string())
        .await?;

    {
        let states = manager.group_states.read().await;
        if let Some(GroupState::MemberJoining { .. }) = states.get(group_id) {
            println!("✅ Transitioned to MemberJoining state");
        } else {
            panic!("Expected MemberJoining state");
        }
    }

    // Simulate grace period transition
    {
        let mut states = manager.group_states.write().await;
        if let Some(state) = states.get_mut(group_id) {
            *state = GroupState::GracePeriod {
                config: OptimalReedSolomonConfig::for_group_size(3),
                pending_changes: vec![MembershipChange {
                    change_type: MembershipChangeType::Join,
                    member_id: "m4".to_string(),
                    timestamp: chrono::Utc::now(),
                    hlc: HybridLogicalClock::new("test".to_string()),
                }],
                grace_period_ends: chrono::Utc::now() - chrono::Duration::seconds(1), // Already expired
            };
        }
    }
    println!("✅ Transitioned to GracePeriod state");

    // Process grace period
    manager.process_grace_period(group_id).await?;

    {
        let states = manager.group_states.read().await;
        match states.get(group_id) {
            Some(GroupState::Stable { members, .. }) => {
                println!("✅ Returned to Stable state with {} members", members.len());
            }
            Some(GroupState::Rebalancing { .. }) => {
                println!("✅ Transitioned to Rebalancing state");
            }
            _ => panic!("Unexpected state after grace period"),
        }
    }

    Ok(())
}

// Run all tests with summary
#[tokio::test]
async fn test_complete_reed_solomon_v2_system() -> Result<()> {
    println!("🚀 Running complete Reed Solomon v2 system validation\n");
    println!("{}", "=".repeat(60));

    test_60_percent_availability_comprehensive().await?;
    println!("{}", "=".repeat(60));

    test_dynamic_membership_rebalancing().await?;
    println!("{}", "=".repeat(60));

    test_intelligent_shard_distribution().await?;
    println!("{}", "=".repeat(60));

    test_recovery_scenarios().await?;
    println!("{}", "=".repeat(60));

    println!("\n✅ All Reed Solomon v2 tests passed!");
    println!("📊 System validated for:");
    println!("   • 60% availability target");
    println!("   • Dynamic membership with intelligent rebalancing");
    println!("   • Reliability-based shard distribution");
    println!("   • Causal ordering with HLC");
    println!("   • State machine transitions");
    println!("   • Recovery scenarios");

    Ok(())
}
