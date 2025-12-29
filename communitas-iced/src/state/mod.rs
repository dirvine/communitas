// Copyright (c) 2025 Saorsa Labs Limited
//
// Dual-licensed under the AGPL-3.0-or-later and a commercial license.
// You may use this file under the terms of the GNU Affero General Public License v3.0 or later.
// For commercial licensing, contact: saorsalabs@gmail.com

//! State management for the Communitas Iced application.
//!
//! This module contains all application state organized by domain.

mod auth;
mod calls;
mod contacts;
mod entities;
mod kanban;
mod messaging;
mod navigation;
mod network;
mod sidebar;

pub use auth::{AuthState, VaultInfo};
pub use calls::{CallInfo, CallParticipant, CallState, CallStatus, MediaDevice, MediaDevices};
pub use contacts::{Contact, ContactStatus};
pub use entities::{Entity, EntityType, MemberRole};
pub use kanban::{CardPriority, KanbanCard, KanbanColumn};
pub use messaging::{ChatMessage, MessageReaction, ThreadState};
pub use navigation::{ActiveView, DetailTab, NavigationState};
pub use network::{BootstrapNode, NetworkInfo, PeerInfo};
pub use sidebar::{SidebarSection, SidebarState};
