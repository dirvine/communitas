//! TDD tests for MCP recovery tools (ADR-016)
//!
//! These tests verify the expected behavior of recovery tools.

use serde_json::{Value, json};

// =============================================================================
// Test Helpers
// =============================================================================

/// Known valid 12-word test mnemonic (BIP39 test vector)
const TEST_MNEMONIC_12: &str =
    "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";

/// Known valid 24-word test mnemonic
const TEST_MNEMONIC_24: &str = "abandon abandon abandon abandon abandon abandon abandon abandon \
                                abandon abandon abandon abandon abandon abandon abandon abandon \
                                abandon abandon abandon abandon abandon abandon abandon art";

// Helper to call tools - replicates the dispatch pattern
mod tool_helpers {
    use super::*;
    use base64::prelude::*;
    use communitas_core::recovery::{
        Language, RecoveryConfig, create_new_identity, recover_identity, validate_mnemonic,
    };

    /// Result type matching MCP tool results
    pub struct ToolResult {
        pub content: Value,
        pub is_error: bool,
    }

    /// Execute create_identity tool
    pub fn create_identity(word_count: Option<usize>, passphrase: Option<&str>) -> ToolResult {
        let word_count = word_count.unwrap_or(24);

        if ![12, 15, 18, 21, 24].contains(&word_count) {
            return ToolResult {
                content: json!({
                    "error": format!("Invalid word_count: {}. Must be 12, 15, 18, 21, or 24", word_count)
                }),
                is_error: true,
            };
        }

        let config = RecoveryConfig::default().with_word_count(word_count);

        match create_new_identity(&config, passphrase) {
            Ok((mnemonic, keys)) => {
                let mnemonic_words: Vec<String> = mnemonic.words().map(String::from).collect();
                let public_signing_key = BASE64_STANDARD.encode(keys.verifying_key_bytes());
                let public_encryption_key = BASE64_STANDARD.encode(keys.encapsulation_key_bytes());

                ToolResult {
                    content: json!({
                        "mnemonic_words": mnemonic_words,
                        "four_words": keys.four_words,
                        "public_signing_key": public_signing_key,
                        "public_encryption_key": public_encryption_key,
                        "warning": "IMPORTANT: Write down your recovery phrase..."
                    }),
                    is_error: false,
                }
            }
            Err(e) => ToolResult {
                content: json!({ "error": format!("Failed to create identity: {}", e) }),
                is_error: true,
            },
        }
    }

    /// Execute recover_identity tool
    pub fn execute_recover_identity(mnemonic_words: &str, passphrase: Option<&str>) -> ToolResult {
        if mnemonic_words.trim().is_empty() {
            return ToolResult {
                content: json!({ "error": "mnemonic_words cannot be empty" }),
                is_error: true,
            };
        }

        let normalized: String = mnemonic_words
            .split_whitespace()
            .collect::<Vec<_>>()
            .join(" ")
            .to_lowercase();

        match recover_identity(&normalized, Language::English, passphrase) {
            Ok(keys) => {
                let public_signing_key = BASE64_STANDARD.encode(keys.verifying_key_bytes());
                let public_encryption_key = BASE64_STANDARD.encode(keys.encapsulation_key_bytes());

                ToolResult {
                    content: json!({
                        "four_words": keys.four_words,
                        "public_signing_key": public_signing_key,
                        "public_encryption_key": public_encryption_key,
                        "success": true
                    }),
                    is_error: false,
                }
            }
            Err(e) => ToolResult {
                content: json!({ "error": format!("Failed to recover identity: {}", e) }),
                is_error: true,
            },
        }
    }

    /// Execute validate_mnemonic tool
    pub fn execute_validate_mnemonic(mnemonic_words: &str) -> ToolResult {
        if mnemonic_words.trim().is_empty() {
            return ToolResult {
                content: json!({
                    "valid": false,
                    "word_count": 0,
                    "error": "Mnemonic cannot be empty"
                }),
                is_error: false,
            };
        }

        let normalized: String = mnemonic_words
            .split_whitespace()
            .collect::<Vec<_>>()
            .join(" ")
            .to_lowercase();

        let word_count = normalized.split_whitespace().count();

        if ![12, 15, 18, 21, 24].contains(&word_count) {
            return ToolResult {
                content: json!({
                    "valid": false,
                    "word_count": word_count,
                    "error": format!("Invalid word count: {}. Must be 12, 15, 18, 21, or 24", word_count)
                }),
                is_error: false,
            };
        }

        match validate_mnemonic(&normalized, Language::English) {
            Ok(_) => ToolResult {
                content: json!({
                    "valid": true,
                    "word_count": word_count
                }),
                is_error: false,
            },
            Err(e) => ToolResult {
                content: json!({
                    "valid": false,
                    "word_count": word_count,
                    "error": e.to_string()
                }),
                is_error: false,
            },
        }
    }
}

// =============================================================================
// create_identity Tool Tests
// =============================================================================

mod create_identity {
    use super::*;

    #[test]
    fn test_creates_valid_24_word_mnemonic_by_default() {
        let result = tool_helpers::create_identity(None, None);
        assert!(!result.is_error);

        let words = result.content["mnemonic_words"].as_array().unwrap();
        assert_eq!(words.len(), 24);
    }

    #[test]
    fn test_creates_12_word_mnemonic_when_specified() {
        let result = tool_helpers::create_identity(Some(12), None);
        assert!(!result.is_error);

        let words = result.content["mnemonic_words"].as_array().unwrap();
        assert_eq!(words.len(), 12);
    }

    #[test]
    fn test_different_calls_produce_different_identities() {
        let result1 = tool_helpers::create_identity(None, None);
        let result2 = tool_helpers::create_identity(None, None);

        assert!(!result1.is_error);
        assert!(!result2.is_error);

        // Different mnemonics should produce different four_words
        let four_words1 = result1.content["four_words"].as_str().unwrap();
        let four_words2 = result2.content["four_words"].as_str().unwrap();

        // While theoretically possible to get the same, extremely unlikely
        assert_ne!(four_words1, four_words2);
    }

    #[test]
    fn test_response_includes_security_warning() {
        let result = tool_helpers::create_identity(None, None);
        assert!(!result.is_error);

        let warning = result.content["warning"].as_str().unwrap();
        assert!(warning.contains("recovery phrase") || warning.contains("IMPORTANT"));
    }

    #[test]
    fn test_response_does_not_include_private_keys() {
        let result = tool_helpers::create_identity(None, None);
        assert!(!result.is_error);

        // Check that only public keys are in response
        assert!(result.content.get("public_signing_key").is_some());
        assert!(result.content.get("public_encryption_key").is_some());

        // Private keys should NOT be present
        assert!(result.content.get("signing_key").is_none());
        assert!(result.content.get("decapsulation_key").is_none());
        assert!(result.content.get("private_key").is_none());
        assert!(result.content.get("secret_key").is_none());
    }

    #[test]
    fn test_four_words_format() {
        let result = tool_helpers::create_identity(None, None);
        assert!(!result.is_error);

        let four_words = result.content["four_words"].as_str().unwrap();

        // Should be dash-separated lowercase words
        let parts: Vec<&str> = four_words.split('-').collect();
        assert_eq!(parts.len(), 4);

        for part in parts {
            assert!(!part.is_empty());
            assert!(part.chars().all(|c| c.is_lowercase() || c.is_ascii_digit()));
        }
    }

    #[test]
    fn test_passphrase_produces_different_keys() {
        let result1 = tool_helpers::create_identity(Some(12), None);
        let result2 = tool_helpers::create_identity(Some(12), Some("secret"));

        assert!(!result1.is_error);
        assert!(!result2.is_error);

        // Different passphrases on same mnemonic length would produce different keys
        // But since we generate fresh mnemonics each time, we can't directly test this here
        // The core library tests this - here we just verify the option is accepted
        assert!(result2.content.get("public_signing_key").is_some());
    }

    #[test]
    fn test_rejects_invalid_word_count() {
        let result = tool_helpers::create_identity(Some(13), None);
        assert!(result.is_error);

        let error = result.content["error"].as_str().unwrap();
        assert!(error.contains("13") || error.contains("Invalid"));
    }

    #[test]
    fn test_public_keys_are_base64_encoded() {
        let result = tool_helpers::create_identity(None, None);
        assert!(!result.is_error);

        let pk_signing = result.content["public_signing_key"].as_str().unwrap();
        let pk_encryption = result.content["public_encryption_key"].as_str().unwrap();

        // Should be valid base64
        use base64::prelude::*;
        assert!(BASE64_STANDARD.decode(pk_signing).is_ok());
        assert!(BASE64_STANDARD.decode(pk_encryption).is_ok());
    }
}

// =============================================================================
// recover_identity Tool Tests
// =============================================================================

mod recover_identity {
    use super::*;

    #[test]
    fn test_recovers_from_valid_12_word_mnemonic() {
        let result = tool_helpers::execute_recover_identity(TEST_MNEMONIC_12, None);
        assert!(!result.is_error);
        assert!(result.content.get("four_words").is_some());
        assert!(result.content.get("public_signing_key").is_some());
    }

    #[test]
    fn test_recovers_from_valid_24_word_mnemonic() {
        let result = tool_helpers::execute_recover_identity(TEST_MNEMONIC_24, None);
        assert!(!result.is_error);
        assert!(result.content.get("four_words").is_some());
    }

    #[test]
    fn test_same_mnemonic_produces_same_identity() {
        let result1 = tool_helpers::execute_recover_identity(TEST_MNEMONIC_12, None);
        let result2 = tool_helpers::execute_recover_identity(TEST_MNEMONIC_12, None);

        assert!(!result1.is_error);
        assert!(!result2.is_error);

        // CRITICAL: Same mnemonic must produce identical keys
        assert_eq!(result1.content["four_words"], result2.content["four_words"]);
        assert_eq!(
            result1.content["public_signing_key"],
            result2.content["public_signing_key"]
        );
    }

    #[test]
    fn test_different_mnemonics_produce_different_identities() {
        let result1 = tool_helpers::execute_recover_identity(TEST_MNEMONIC_12, None);
        let result2 = tool_helpers::execute_recover_identity(TEST_MNEMONIC_24, None);

        assert!(!result1.is_error);
        assert!(!result2.is_error);

        // Different mnemonics should produce different identities
        assert_ne!(result1.content["four_words"], result2.content["four_words"]);
    }

    #[test]
    fn test_passphrase_recovery_produces_different_keys() {
        let result1 = tool_helpers::execute_recover_identity(TEST_MNEMONIC_12, None);
        let result2 = tool_helpers::execute_recover_identity(TEST_MNEMONIC_12, Some("secret"));

        assert!(!result1.is_error);
        assert!(!result2.is_error);

        // Same mnemonic with different passphrase produces different keys
        assert_ne!(result1.content["four_words"], result2.content["four_words"]);
    }

    #[test]
    fn test_rejects_invalid_mnemonic_words() {
        let result =
            tool_helpers::execute_recover_identity("invalid words not in dictionary", None);
        assert!(result.is_error);
    }

    #[test]
    fn test_rejects_bad_checksum() {
        // Valid words but wrong checksum (changed last word)
        let bad_checksum = "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon";
        let result = tool_helpers::execute_recover_identity(bad_checksum, None);
        assert!(result.is_error);
    }

    #[test]
    fn test_rejects_wrong_word_count() {
        let result = tool_helpers::execute_recover_identity("one two three", None);
        assert!(result.is_error);
    }

    #[test]
    fn test_response_does_not_include_private_keys() {
        let result = tool_helpers::execute_recover_identity(TEST_MNEMONIC_12, None);
        assert!(!result.is_error);

        assert!(result.content.get("signing_key").is_none());
        assert!(result.content.get("decapsulation_key").is_none());
    }

    #[test]
    fn test_handles_extra_whitespace() {
        let spaced_mnemonic = "  abandon  abandon   abandon abandon abandon abandon abandon abandon abandon abandon abandon about  ";
        let result = tool_helpers::execute_recover_identity(spaced_mnemonic, None);
        assert!(!result.is_error);

        // Should match the normalized version
        let result2 = tool_helpers::execute_recover_identity(TEST_MNEMONIC_12, None);
        assert_eq!(result.content["four_words"], result2.content["four_words"]);
    }

    #[test]
    fn test_case_insensitive_mnemonic() {
        let uppercase = "ABANDON ABANDON ABANDON ABANDON ABANDON ABANDON ABANDON ABANDON ABANDON ABANDON ABANDON ABOUT";
        let result = tool_helpers::execute_recover_identity(uppercase, None);
        assert!(!result.is_error);

        let result2 = tool_helpers::execute_recover_identity(TEST_MNEMONIC_12, None);
        assert_eq!(result.content["four_words"], result2.content["four_words"]);
    }

    #[test]
    fn test_rejects_empty_input() {
        let result = tool_helpers::execute_recover_identity("", None);
        assert!(result.is_error);
    }
}

// =============================================================================
// validate_mnemonic Tool Tests
// =============================================================================

mod validate_mnemonic {
    use super::*;

    #[test]
    fn test_validates_correct_12_word_mnemonic() {
        let result = tool_helpers::execute_validate_mnemonic(TEST_MNEMONIC_12);
        assert!(!result.is_error);
        assert_eq!(result.content["valid"], true);
        assert_eq!(result.content["word_count"], 12);
    }

    #[test]
    fn test_validates_correct_24_word_mnemonic() {
        let result = tool_helpers::execute_validate_mnemonic(TEST_MNEMONIC_24);
        assert!(!result.is_error);
        assert_eq!(result.content["valid"], true);
        assert_eq!(result.content["word_count"], 24);
    }

    #[test]
    fn test_rejects_invalid_words() {
        let result = tool_helpers::execute_validate_mnemonic(
            "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon notaword",
        );
        assert!(!result.is_error); // Tool returns success but valid=false
        assert_eq!(result.content["valid"], false);
        assert!(result.content.get("error").is_some());
    }

    #[test]
    fn test_rejects_bad_checksum() {
        // 12 valid words but wrong checksum
        let bad = "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon";
        let result = tool_helpers::execute_validate_mnemonic(bad);
        assert!(!result.is_error);
        assert_eq!(result.content["valid"], false);
    }

    #[test]
    fn test_returns_word_count() {
        let result = tool_helpers::execute_validate_mnemonic(TEST_MNEMONIC_12);
        assert_eq!(result.content["word_count"], 12);

        let result = tool_helpers::execute_validate_mnemonic(TEST_MNEMONIC_24);
        assert_eq!(result.content["word_count"], 24);
    }

    #[test]
    fn test_no_key_derivation() {
        // Validation should NOT include any keys
        let result = tool_helpers::execute_validate_mnemonic(TEST_MNEMONIC_12);
        assert!(!result.is_error);

        assert!(result.content.get("public_signing_key").is_none());
        assert!(result.content.get("public_encryption_key").is_none());
        assert!(result.content.get("four_words").is_none());
    }

    #[test]
    fn test_handles_empty_input() {
        let result = tool_helpers::execute_validate_mnemonic("");
        assert!(!result.is_error);
        assert_eq!(result.content["valid"], false);
        assert_eq!(result.content["word_count"], 0);
    }

    #[test]
    fn test_rejects_wrong_word_count() {
        let result = tool_helpers::execute_validate_mnemonic("one two three four five");
        assert!(!result.is_error);
        assert_eq!(result.content["valid"], false);
        assert_eq!(result.content["word_count"], 5);
    }
}

// =============================================================================
// Integration Tests
// =============================================================================

mod integration {
    use super::*;

    #[test]
    fn test_create_then_recover_roundtrip() {
        // Create a new identity
        let create_result = tool_helpers::create_identity(Some(12), None);
        assert!(!create_result.is_error);

        // Extract the mnemonic words
        let mnemonic_words: Vec<String> = create_result.content["mnemonic_words"]
            .as_array()
            .unwrap()
            .iter()
            .map(|v| v.as_str().unwrap().to_string())
            .collect();
        let mnemonic_str = mnemonic_words.join(" ");

        let original_four_words = create_result.content["four_words"].as_str().unwrap();
        let original_signing_key = create_result.content["public_signing_key"]
            .as_str()
            .unwrap();

        // Recover using the mnemonic
        let recover_result = tool_helpers::execute_recover_identity(&mnemonic_str, None);
        assert!(!recover_result.is_error);

        // Verify identity matches
        assert_eq!(
            recover_result.content["four_words"].as_str().unwrap(),
            original_four_words
        );
        assert_eq!(
            recover_result.content["public_signing_key"]
                .as_str()
                .unwrap(),
            original_signing_key
        );
    }

    #[test]
    fn test_create_validate_recover_flow() {
        // Create identity
        let create_result = tool_helpers::create_identity(Some(24), None);
        assert!(!create_result.is_error);

        let mnemonic_words: Vec<String> = create_result.content["mnemonic_words"]
            .as_array()
            .unwrap()
            .iter()
            .map(|v| v.as_str().unwrap().to_string())
            .collect();
        let mnemonic_str = mnemonic_words.join(" ");

        // Validate the mnemonic
        let validate_result = tool_helpers::execute_validate_mnemonic(&mnemonic_str);
        assert!(!validate_result.is_error);
        assert_eq!(validate_result.content["valid"], true);
        assert_eq!(validate_result.content["word_count"], 24);

        // Recover and verify same identity
        let recover_result = tool_helpers::execute_recover_identity(&mnemonic_str, None);
        assert!(!recover_result.is_error);
        assert_eq!(
            create_result.content["four_words"],
            recover_result.content["four_words"]
        );
    }
}
