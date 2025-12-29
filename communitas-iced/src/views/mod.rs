// Copyright (c) 2025 Saorsa Labs Limited
//
// Dual-licensed under the AGPL-3.0-or-later and a commercial license.
// You may use this file under the terms of the GNU Affero General Public License v3.0 or later.
// For commercial licensing, contact: saorsalabs@gmail.com

//! View modules for the Communitas GUI.
//!
//! Each view module provides functions that render different parts of the UI.

pub mod authentication;
pub mod main_layout;

pub use authentication::view_authentication;
pub use main_layout::{view_main, ModalFormState};
