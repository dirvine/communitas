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

fn main() {
    // Skip Tauri build when producing the headless binary to avoid GUI deps.
    // Controlled via env var set in CI and container builds.
    if std::env::var("COMMUNITAS_SKIP_TAURI_BUILD").is_ok() {
        return;
    }
    tauri_build::build();
}
