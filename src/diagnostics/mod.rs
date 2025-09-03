// Copyright (c) 2025 Saorsa Labs Limited

// This file is part of the Saorsa P2P network.

// Licensed under the AGPL-3.0 license:
// <https://www.gnu.org/licenses/agpl-3.0.html>

// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
// GNU Affero General Public License for more details.

// You should have received a copy of the GNU Affero General Public License
// along with this program. If not, see <https://www.gnu.org/licenses/>.

//! Network diagnostics and monitoring

mod engine;
mod metrics;

pub use engine::DiagnosticsEngine;
pub use metrics::{NetworkHealth, NetworkMetrics, NetworkStats, PeerMetrics, StorageMetrics};

/// NAT type detected
#[derive(Debug, Clone, Copy, serde::Serialize, serde::Deserialize, Default)]
pub enum NatType {
    /// No NAT - direct connection
    None,
    /// Full cone NAT
    FullCone,
    /// Restricted cone NAT
    RestrictedCone,
    /// Port restricted cone NAT
    PortRestrictedCone,
    /// Symmetric NAT
    Symmetric,
    /// Unknown/detecting
    #[default]
    Unknown,
}
