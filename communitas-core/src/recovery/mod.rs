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

pub mod error;
pub mod mnemonic;

pub use error::{RecoveryError, RecoveryResult};
pub use mnemonic::{
    RecoveryConfig, generate_recovery_mnemonic, mnemonic_to_words, validate_mnemonic,
};
