//! Integration tests and examples for the DHT identity system
//!
//! This module provides comprehensive integration tests that demonstrate
//! the complete DHT identity system working together.

use crate::dht_identity::{blobs::*, key_derivation::*, records::*, storage::ResolvedIdentity};
use std::time::{SystemTime, UNIX_EPOCH};

/// Complete example of creating and validating an identity
pub async fn create_complete_identity_example() -> Result<ResolvedIdentity, String> {
    let four_words = NormalizedFourWords::new("ocean forest moon star")
        .map_err(|e| format!("Four words validation failed: {}", e))?;

    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis() as u64;

    // Step 1: Generate test ML-DSA and ML-KEM keys
    let ml_dsa_key = vec![1u8; 1952]; // Mock ML-DSA-65 public key
    let ml_kem_key = vec![2u8; 1184]; // Mock ML-KEM-768 public key

    // Step 2: Create transport key entries
    let transport_spki = b"test_ed25519_spki_data".to_vec();
    let transport_keys = vec![TransportKeyEntry {
        spki: transport_spki.clone(),
        algorithm: "Ed25519".to_string(),
        peer_id: *blake3::hash(&transport_spki).as_bytes(),
    }];

    // Step 3: Create identity descriptor blob
    let mut descriptor = IdentityDescriptorBlob::new(
        four_words.as_str().to_string(),
        [0u8; 32], // root_digest (will be updated)
        ml_dsa_key.clone(),
        ml_kem_key.clone(),
        transport_keys,
        "Ocean Forest Moon Star".to_string(),
    );

    // Step 4: Create identity root record
    let descriptor_cid = descriptor
        .content_address()
        .map_err(|e| format!("Failed to compute descriptor CID: {}", e))?;

    let mut root_record = IdentityRootRecord::new(
        1, // sequence
        timestamp,
        *blake3::hash(four_words.as_str().as_bytes()).as_bytes(), // identity_hash
        *blake3::hash(&ml_dsa_key).as_bytes(),                    // ml_dsa_key_hash
        *blake3::hash(&ml_kem_key).as_bytes(),                    // ml_kem_key_hash
        *blake3::hash(&transport_spki).as_bytes(),                // transport_id
        descriptor_cid,
    );

    // Step 5: Update descriptor with correct root core digest (breaks circular dependency)
    descriptor.root_digest = root_record.core_hash();
    let final_descriptor_cid = descriptor
        .content_address()
        .map_err(|e| format!("Failed to compute final descriptor CID: {}", e))?;
    root_record.descriptor_cid = final_descriptor_cid;

    // Step 6: Create connection information (NAT traversal)
    let rendezvous_nodes = vec![
        RendezvousEntry {
            address: "bootstrap1.communitas.network:9000".to_string(),
            priority: 1,
        },
        RendezvousEntry {
            address: "bootstrap2.communitas.network:9000".to_string(),
            priority: 2,
        },
    ];

    let connection_blob = ConnectionBlob::new(
        *blake3::hash(&transport_spki).as_bytes(), // peer_transport_id
        rendezvous_nodes,
        timestamp,
        3600, // 1 hour TTL
    );

    let connection_cid = connection_blob
        .content_address()
        .map_err(|e| format!("Failed to compute connection CID: {}", e))?;

    let connection_record = ConnectionRecord::new(
        1, // sequence
        timestamp,
        *blake3::hash(&transport_spki).as_bytes(), // transport_id
        connection_cid,
        3600,
    );

    // Update root record with connection
    root_record.connection_cid = Some(connection_cid);

    // Step 7: Create website content (example)
    let pages = vec![
        PageEntry {
            path: "/index.html".to_string(),
            content_id: [10u8; 32],
            size_bytes: 2048,
            mime_type: "text/html".to_string(),
            content_hash: [100u8; 32],
        },
        PageEntry {
            path: "/about.html".to_string(),
            content_id: [11u8; 32],
            size_bytes: 1024,
            mime_type: "text/html".to_string(),
            content_hash: [101u8; 32],
        },
        PageEntry {
            path: "/style.css".to_string(),
            content_id: [12u8; 32],
            size_bytes: 512,
            mime_type: "text/css".to_string(),
            content_hash: [102u8; 32],
        },
    ];

    let site_manifest = SiteManifestBlob::new("/index.html".to_string(), pages)
        .map_err(|e| format!("Failed to create site manifest: {}", e))?;

    let site_cid = site_manifest
        .content_address()
        .map_err(|e| format!("Failed to compute site CID: {}", e))?;

    let site_record = SiteManifestRecord::new(
        1, // sequence
        timestamp, site_cid, 7200, // 2 hours TTL
    );

    // Update root record with site
    root_record.site_cid = Some(site_cid);
    root_record.sequence = 2; // Increment for the update

    // Step 8: Update descriptor with final root core digest after all changes
    descriptor.root_digest = root_record.core_hash();
    let final_final_descriptor_cid = descriptor
        .content_address()
        .map_err(|e| format!("Failed to compute final descriptor CID: {}", e))?;
    root_record.descriptor_cid = final_final_descriptor_cid;

    // Step 9: Validate record sizes
    root_record
        .validate_size()
        .map_err(|e| format!("Root record too large: {}", e))?;
    connection_record
        .validate_size()
        .map_err(|e| format!("Connection record too large: {}", e))?;
    site_record
        .validate_size()
        .map_err(|e| format!("Site record too large: {}", e))?;

    // Step 10: Create the complete resolved identity
    Ok(ResolvedIdentity {
        four_words,
        root_record,
        descriptor,
        connection_info: Some((connection_record, connection_blob)),
        site_info: Some((site_record, site_manifest)),
    })
}

/// Demonstrate the complete identity lifecycle
pub async fn demonstrate_identity_lifecycle() -> Result<(), String> {
    println!("🔧 Creating complete identity with all components...");

    // Create complete identity
    let identity = create_complete_identity_example().await?;

    println!("✅ Identity created successfully:");
    println!("   Four-words: {}", identity.four_words.as_str());
    println!("   Sequence: {}", identity.root_record.sequence);
    println!(
        "   Display name: {}",
        identity.descriptor.preferred_display_name
    );

    // Show DHT keys
    let id_key = derive_identity_key(&identity.four_words);
    let conn_key = derive_connection_key(&identity.four_words);
    let site_key = derive_site_key(&identity.four_words);

    println!("📂 DHT storage locations:");
    println!("   Identity: {:02x?}...", &id_key[..8]);
    println!("   Connection: {:02x?}...", &conn_key[..8]);
    println!("   Site: {:02x?}...", &site_key[..8]);

    // Show record sizes
    let root_size = identity.root_record.to_cbor().unwrap().len();
    if let Some((conn_record, _)) = &identity.connection_info {
        let conn_size = conn_record.to_cbor().unwrap().len();
        println!("📊 DHT record sizes:");
        println!("   Root record: {} bytes", root_size);
        println!("   Connection record: {} bytes", conn_size);
    }

    if let Some((site_record, site_manifest)) = &identity.site_info {
        let site_size = site_record.to_cbor().unwrap().len();
        println!("   Site record: {} bytes", site_size);
        println!(
            "   Total website content: {} bytes",
            site_manifest.total_size()
        );
    }

    // Show content addresses
    let descriptor_cid = identity.descriptor.content_address().unwrap();
    println!("🔗 Content addresses:");
    println!("   Descriptor: {:02x?}...", &descriptor_cid[..8]);

    if let Some((_, conn_blob)) = &identity.connection_info {
        let conn_cid = conn_blob.content_address().unwrap();
        println!("   Connection blob: {:02x?}...", &conn_cid[..8]);
    }

    if let Some((_, site_manifest)) = &identity.site_info {
        let site_cid = site_manifest.content_address().unwrap();
        println!("   Site manifest: {:02x?}...", &site_cid[..8]);
    }

    println!("🎉 Complete identity system demonstration successful!");

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_complete_identity_creation() {
        let identity = create_complete_identity_example()
            .await
            .expect("Should create complete identity");

        // Verify all components are present
        assert_eq!(identity.four_words.as_str(), "ocean-forest-moon-star");
        assert!(identity.connection_info.is_some());
        assert!(identity.site_info.is_some());

        // Verify root record
        assert_eq!(identity.root_record.sequence, 2); // Updated after adding site
        assert!(identity.root_record.connection_cid.is_some());
        assert!(identity.root_record.site_cid.is_some());

        // Verify descriptor
        assert_eq!(identity.descriptor.four_words, "ocean-forest-moon-star");
        assert_eq!(
            identity.descriptor.preferred_display_name,
            "Ocean Forest Moon Star"
        );
        assert!(!identity.descriptor.ml_dsa_public_key.is_empty());
        assert!(!identity.descriptor.ml_kem_public_key.is_empty());

        // Verify connection info
        if let Some((conn_record, conn_blob)) = &identity.connection_info {
            assert_eq!(conn_record.sequence, 1);
            assert_eq!(conn_blob.rendezvous_nodes.len(), 2);
            assert_eq!(
                conn_blob.rendezvous_nodes[0].address,
                "bootstrap1.communitas.network:9000"
            );
        }

        // Verify site info
        if let Some((site_record, site_manifest)) = &identity.site_info {
            assert_eq!(site_record.sequence, 1);
            assert_eq!(site_manifest.pages.len(), 3);
            assert_eq!(site_manifest.index_file, "/index.html");
            assert_eq!(site_manifest.total_size(), 3584); // 2048 + 1024 + 512
        }
    }

    #[tokio::test]
    async fn test_identity_key_derivation_complete() {
        let four_words = NormalizedFourWords::new("ocean forest moon star").unwrap();

        let id_key = derive_identity_key(&four_words);
        let conn_key = derive_connection_key(&four_words);
        let site_key = derive_site_key(&four_words);

        // Verify all keys are different
        assert_ne!(id_key, conn_key);
        assert_ne!(id_key, site_key);
        assert_ne!(conn_key, site_key);

        // Verify keys are deterministic
        assert_eq!(id_key, derive_identity_key(&four_words));
        assert_eq!(conn_key, derive_connection_key(&four_words));
        assert_eq!(site_key, derive_site_key(&four_words));
    }

    #[tokio::test]
    async fn test_content_addressing_consistency() {
        // Create two identical descriptors
        let descriptor1 = IdentityDescriptorBlob::new(
            "test-identity".to_string(),
            [1u8; 32],
            vec![2u8; 1952],
            vec![3u8; 1184],
            vec![],
            "Test".to_string(),
        );

        let descriptor2 = IdentityDescriptorBlob::new(
            "test-identity".to_string(),
            [1u8; 32],
            vec![2u8; 1952],
            vec![3u8; 1184],
            vec![],
            "Test".to_string(),
        );

        // Should have same content address
        let cid1 = descriptor1.content_address().unwrap();
        let cid2 = descriptor2.content_address().unwrap();
        assert_eq!(cid1, cid2);

        // Different content should have different CID
        let descriptor3 = IdentityDescriptorBlob::new(
            "different-identity".to_string(),
            [1u8; 32],
            vec![2u8; 1952],
            vec![3u8; 1184],
            vec![],
            "Test".to_string(),
        );

        let cid3 = descriptor3.content_address().unwrap();
        assert_ne!(cid1, cid3);
    }

    #[tokio::test]
    async fn test_dht_record_size_compliance() {
        let identity = create_complete_identity_example()
            .await
            .expect("Should create identity");

        // Verify all DHT records are within size limits
        let root_size = identity.root_record.to_cbor().unwrap().len();
        assert!(
            root_size <= super::super::DHT_RECORD_MAX_SIZE,
            "Root record too large: {} bytes",
            root_size
        );

        if let Some((conn_record, _)) = &identity.connection_info {
            let conn_size = conn_record.to_cbor().unwrap().len();
            assert!(
                conn_size <= super::super::DHT_RECORD_MAX_SIZE,
                "Connection record too large: {} bytes",
                conn_size
            );
        }

        if let Some((site_record, _)) = &identity.site_info {
            let site_size = site_record.to_cbor().unwrap().len();
            assert!(
                site_size <= super::super::DHT_RECORD_MAX_SIZE,
                "Site record too large: {} bytes",
                site_size
            );
        }

        println!("✅ All DHT records within 512B limit");
        println!("   Root: {} bytes", root_size);
        if let Some((conn_record, _)) = &identity.connection_info {
            println!(
                "   Connection: {} bytes",
                conn_record.to_cbor().unwrap().len()
            );
        }
        if let Some((site_record, _)) = &identity.site_info {
            println!("   Site: {} bytes", site_record.to_cbor().unwrap().len());
        }
    }

    #[tokio::test]
    async fn test_website_content_limit() {
        // Test that site manifest enforces 5MB limit
        let large_pages = vec![
            PageEntry {
                path: "/large1.bin".to_string(),
                content_id: [1u8; 32],
                size_bytes: 3 * 1024 * 1024, // 3MB
                mime_type: "application/octet-stream".to_string(),
                content_hash: [10u8; 32],
            },
            PageEntry {
                path: "/large2.bin".to_string(),
                content_id: [2u8; 32],
                size_bytes: 3 * 1024 * 1024, // 3MB (total would be 6MB)
                mime_type: "application/octet-stream".to_string(),
                content_hash: [20u8; 32],
            },
        ];

        let result = SiteManifestBlob::new("/large1.bin".to_string(), large_pages);
        assert!(result.is_err(), "Should fail when exceeding 5MB limit");
        assert!(result.unwrap_err().contains("exceeds limit"));

        // Test valid size
        let valid_pages = vec![
            PageEntry {
                path: "/index.html".to_string(),
                content_id: [1u8; 32],
                size_bytes: 2 * 1024 * 1024, // 2MB
                mime_type: "text/html".to_string(),
                content_hash: [10u8; 32],
            },
            PageEntry {
                path: "/assets.js".to_string(),
                content_id: [2u8; 32],
                size_bytes: 2 * 1024 * 1024, // 2MB (total 4MB)
                mime_type: "application/javascript".to_string(),
                content_hash: [20u8; 32],
            },
        ];

        let manifest = SiteManifestBlob::new("/index.html".to_string(), valid_pages)
            .expect("Should create manifest within limits");

        assert_eq!(manifest.total_size(), 4 * 1024 * 1024); // 4MB
        manifest.validate().expect("Should be valid");
    }

    #[tokio::test]
    async fn test_nat_traversal_integration() {
        let identity = create_complete_identity_example()
            .await
            .expect("Should create identity");

        if let Some((_, connection_blob)) = &identity.connection_info {
            // Verify NAT traversal setup
            assert_eq!(connection_blob.rendezvous_nodes.len(), 2);
            assert!(connection_blob.policy.allow_relays);
            assert!(connection_blob.policy.path_migration);

            // Verify rendezvous nodes are properly configured
            let node1 = &connection_blob.rendezvous_nodes[0];
            assert_eq!(node1.address, "bootstrap1.communitas.network:9000");
            assert_eq!(node1.priority, 1);

            let node2 = &connection_blob.rendezvous_nodes[1];
            assert_eq!(node2.address, "bootstrap2.communitas.network:9000");
            assert_eq!(node2.priority, 2);

            // Verify transport ID consistency
            let transport_spki = b"test_ed25519_spki_data".to_vec();
            let expected_transport_id = *blake3::hash(&transport_spki).as_bytes();
            assert_eq!(connection_blob.peer_transport_id, expected_transport_id);
        } else {
            panic!("Connection info should be present");
        }
    }

    #[tokio::test]
    async fn test_identity_hash_binding() {
        let identity = create_complete_identity_example()
            .await
            .expect("Should create identity");

        // Verify identity hash correctly binds four-words to the record
        let expected_hash = *blake3::hash(identity.four_words.as_str().as_bytes()).as_bytes();
        assert_eq!(identity.root_record.identity_hash, expected_hash);

        // Verify descriptor references the correct four-words
        assert_eq!(identity.descriptor.four_words, identity.four_words.as_str());

        // Verify root core digest binding (excluding descriptor_cid)
        let expected_root_core_digest = identity.root_record.core_hash();
        assert_eq!(identity.descriptor.root_digest, expected_root_core_digest);
    }

    #[tokio::test]
    async fn test_pqc_key_hash_consistency() {
        let identity = create_complete_identity_example()
            .await
            .expect("Should create identity");

        // Verify key hashes match the actual keys
        let expected_ml_dsa_hash = *blake3::hash(&identity.descriptor.ml_dsa_public_key).as_bytes();
        let expected_ml_kem_hash = *blake3::hash(&identity.descriptor.ml_kem_public_key).as_bytes();

        assert_eq!(identity.root_record.ml_dsa_key_hash, expected_ml_dsa_hash);
        assert_eq!(identity.root_record.ml_kem_key_hash, expected_ml_kem_hash);
    }

    #[tokio::test]
    async fn test_demonstrate_lifecycle() {
        // This test runs the demonstration function
        let result = demonstrate_identity_lifecycle().await;
        assert!(result.is_ok(), "Lifecycle demonstration should succeed");
    }
}
