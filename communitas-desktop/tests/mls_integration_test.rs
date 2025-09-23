// Copyright (c) 2025 Saorsa Labs Limited
//
// Dual-licensed under the AGPL-3.0-or-later and a commercial license.
// You may use this file under the terms of the GNU Affero General Public License v3.0 or later.
// For commercial licensing, contact: saorsalabs@gmail.com
//
// See the LICENSE-AGPL-3.0 and LICENSE-COMMERCIAL.md files for details.

#[cfg(test)]
mod tests {
    use communitas_core::messaging::{MlsClient, MlsConfig};
    use saorsa_mls::GroupId;

    #[tokio::test]
    async fn test_mls_client_creation() {
        let config = MlsConfig {
            enable_pqc: true,
            max_epochs: 1000,
            key_rotation_interval: 100,
            ..Default::default()
        };

        let client = MlsClient::new(config).await;
        assert!(client.is_ok(), "MLS client should be created successfully");
    }

    #[tokio::test]
    async fn test_group_id_generation() {
        let group_id = GroupId::generate();
        assert!(!group_id.to_string().is_empty(), "GroupId should generate non-empty string");
    }

    #[tokio::test]
    async fn test_mls_config_creation() {
        let config = MlsConfig {
            enable_pqc: true,
            max_epochs: 1000,
            key_rotation_interval: 100,
            ..Default::default()
        };

        assert!(config.enable_pqc, "PQC should be enabled by default");
        assert_eq!(config.max_epochs, 1000, "Max epochs should be set correctly");
        assert_eq!(config.key_rotation_interval, 100, "Key rotation interval should be set correctly");
    }

    #[tokio::test]
    async fn test_group_id_string_conversion() {
        let group_id = GroupId::generate();
        let group_id_string = group_id.to_string();
        assert!(!group_id_string.is_empty(), "GroupId string representation should not be empty");
    }
}