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

//! Test harness implementation

use super::{TestMetrics, TestResult, TestScenario};
use anyhow::Result;
use std::fmt;

/// Test harness for running scenarios
pub struct TestHarness {
    /// Test scenarios
    scenarios: Vec<Box<dyn TestScenario>>,
    /// Test nodes
    test_nodes: Vec<TestNode>,
}

impl fmt::Debug for TestHarness {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("TestHarness")
            .field("scenarios_count", &self.scenarios.len())
            .field("test_nodes", &self.test_nodes)
            .finish()
    }
}

/// Test node instance
#[derive(Debug)]
pub struct TestNode {
    /// Node ID
    #[allow(dead_code)]
    pub id: String,
    /// Four-word address
    #[allow(dead_code)]
    pub address: String,
    // TODO: Add actual P2P node instance
}

impl TestHarness {
    /// Create new test harness
    pub fn new() -> Self {
        Self {
            scenarios: Vec::new(),
            test_nodes: Vec::new(),
        }
    }

    /// Add a test scenario
    pub fn add_scenario(&mut self, scenario: Box<dyn TestScenario>) {
        self.scenarios.push(scenario);
    }

    /// Spawn test nodes
    pub async fn spawn_nodes(&mut self, count: usize) -> Result<()> {
        for i in 0..count {
            let node = TestNode {
                id: format!("test-node-{}", i),
                address: format!("test-{}-{}-{}", i, i, i),
            };
            self.test_nodes.push(node);
        }
        Ok(())
    }

    /// Run all scenarios
    pub async fn run_all(&mut self) -> Result<Vec<TestResult>> {
        let mut results = Vec::new();

        for scenario in &mut self.scenarios {
            println!("Running scenario: {}", scenario.name());

            // Setup
            scenario.setup()?;

            // Execute
            let start = std::time::Instant::now();
            let result = match scenario.execute() {
                Ok(mut result) => {
                    result.duration_ms = start.elapsed().as_millis() as u64;
                    result
                }
                Err(e) => TestResult {
                    passed: false,
                    duration_ms: start.elapsed().as_millis() as u64,
                    error: Some(e.to_string()),
                    metrics: TestMetrics::default(),
                },
            };

            // Teardown
            let _ = scenario.teardown();

            results.push(result);
        }

        Ok(results)
    }
}
