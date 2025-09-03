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

//! Network integration with P2P Foundation

mod integration;

pub use integration::NetworkIntegration;

/// Network-related errors
#[derive(Debug, thiserror::Error)]
pub enum NetworkError {
    /// Bootstrap connection failed
    #[error("Failed to connect to bootstrap node: {0}")]
    BootstrapFailed(String),

    /// DHT operation failed
    #[error("DHT operation failed: {0}")]
    DhtError(String),

    /// Transport error
    #[error("Transport error: {0}")]
    TransportError(String),
}
