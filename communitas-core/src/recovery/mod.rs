// Copyright (c) 2025 Saorsa Labs Limited
//
// This file is part of the Saorsa P2P network.
//
// Licensed under the AGPL-3.0 license:
// <https://www.gnu.org/licenses/agpl-3.0.html>

//! Recovery module for vault backup and restoration (ADR-016).
//!
//! This module provides BIP-39 compatible mnemonic phrase generation and
//! validation for vault recovery. Users can backup their vault with a
//! 24-word recovery phrase and restore it on any device.
//!
//! ## Key Derivation
//!
//! The recovery system uses deterministic key derivation from BIP39 mnemonics:
//! - ML-DSA-87 signing keys for identity authentication (Level 5 PQC, 192-bit quantum security)
//! - ML-KEM-768 encapsulation keys for message encryption
//! - Four-word identity derived from the public signing key
//!
//! See [`derive_identity_keys`] for the key derivation function.

pub mod error;
pub mod keys;
pub mod mnemonic;

pub use error::{RecoveryError, RecoveryResult};
pub use keys::{IdentityKeys, create_new_identity, derive_identity_keys, recover_identity};
pub use mnemonic::{
    RecoveryConfig, generate_recovery_mnemonic, mnemonic_to_words, validate_mnemonic,
};

// Re-export bip39::Language for consumers who need to specify language
pub use bip39::Language;
