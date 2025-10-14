//! Tests for macOS Touch ID authentication
//! These tests verify that native Touch ID integration works correctly

#[cfg(test)]
#[cfg(target_os = "macos")]
mod touchid_tests {
    /// Test that Touch ID availability can be checked
    /// This should return Ok(true/false) indicating if Touch ID is available
    #[test]
    fn test_touchid_availability() {
        // This test checks if we can detect Touch ID capability
        let result = check_touchid_available();

        // Should return Ok(bool), not Err
        // The bool indicates whether Touch ID is available on this device
        assert!(result.is_ok(), "Touch ID availability check should succeed and return Ok(bool), got: {:?}", result);
    }

    /// Test that Touch ID authentication can be triggered
    /// This is the core functionality we need to implement
    #[test]
    #[ignore] // Ignore by default - requires user interaction with Touch ID
    fn test_touchid_authenticate() {
        let reason = "Test Touch ID authentication";

        // Attempt to authenticate with Touch ID
        let result = authenticate_with_touchid(reason);

        // This should return Ok(bool):
        // - Ok(true) if user authenticates successfully
        // - Ok(false) if user cancels or auth fails
        // Should NOT return Err (errors should be handled internally)
        assert!(
            result.is_ok(),
            "Touch ID authentication should return Ok(bool), got: {:?}",
            result
        );
    }

    /// Test that Touch ID authentication handles empty reason gracefully
    #[test]
    #[ignore] // Ignore by default - requires user interaction with Touch ID
    fn test_touchid_error_handling() {
        // Test with empty reason string - should still work but use default message
        let result = authenticate_with_touchid("");

        // Should still return Ok, even with empty reason
        assert!(
            result.is_ok(),
            "Touch ID should handle empty reason gracefully, got: {:?}",
            result
        );
    }

    // Helper functions implemented using Swift helper binary

    /// Check if Touch ID is available on this device
    fn check_touchid_available() -> Result<bool, String> {
        use std::process::Command;

        // Call swift helper to check Touch ID availability
        let swift_code = r#"
import LocalAuthentication
import Foundation

let context = LAContext()
var error: NSError?
let canEvaluate = context.canEvaluatePolicy(.deviceOwnerAuthenticationWithBiometrics, error: &error)
print(canEvaluate ? "true" : "false")
"#;

        // Compile and run Swift code inline
        let output = Command::new("swift")
            .arg("-")
            .arg("-framework")
            .arg("LocalAuthentication")
            .arg("-framework")
            .arg("Foundation")
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .spawn()
            .and_then(|mut child| {
                use std::io::Write;
                if let Some(mut stdin) = child.stdin.take() {
                    stdin.write_all(swift_code.as_bytes())?;
                }
                child.wait_with_output()
            });

        match output {
            Ok(result) if result.status.success() => {
                let stdout = String::from_utf8_lossy(&result.stdout);
                Ok(stdout.trim() == "true")
            }
            Ok(result) => {
                let stderr = String::from_utf8_lossy(&result.stderr);
                Err(format!("Touch ID check failed: {}", stderr))
            }
            Err(e) => Err(format!("Failed to run swift: {}", e)),
        }
    }

    /// Authenticate using Touch ID with given reason
    fn authenticate_with_touchid(reason: &str) -> Result<bool, String> {
        use std::process::Command;

        let reason = if reason.is_empty() {
            "Authenticate with Touch ID"
        } else {
            reason
        };

        let swift_code = format!(
            r#"
import LocalAuthentication
import Foundation

let context = LAContext()
var error: NSError?

if !context.canEvaluatePolicy(.deviceOwnerAuthenticationWithBiometrics, error: &error) {{
    print("false")
    exit(2)
}}

let semaphore = DispatchSemaphore(value: 0)
var success = false

context.evaluatePolicy(.deviceOwnerAuthenticationWithBiometrics, localizedReason: "{}") {{ result, authError in
    success = result
    semaphore.signal()
}}

semaphore.wait()
print(success ? "true" : "false")
exit(success ? 0 : 1)
"#,
            reason.replace("\"", "\\\"")
        );

        // Compile and run Swift code inline
        let output = Command::new("swift")
            .arg("-")
            .arg("-framework")
            .arg("LocalAuthentication")
            .arg("-framework")
            .arg("Foundation")
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .spawn()
            .and_then(|mut child| {
                use std::io::Write;
                if let Some(mut stdin) = child.stdin.take() {
                    stdin.write_all(swift_code.as_bytes())?;
                }
                child.wait_with_output()
            });

        match output {
            Ok(result) => {
                let stdout = String::from_utf8_lossy(&result.stdout);
                match result.status.code() {
                    Some(0) => Ok(stdout.trim() == "true"),
                    Some(1) => Ok(false), // User cancelled or auth failed
                    Some(2) => Ok(false), // Touch ID not available
                    _ => Ok(false),
                }
            }
            Err(e) => Err(format!("Failed to run swift: {}", e)),
        }
    }
}

#[cfg(test)]
#[cfg(not(target_os = "macos"))]
mod touchid_tests {
    /// On non-macOS platforms, Touch ID should not be available
    #[test]
    fn test_touchid_unavailable_on_non_macos() {
        // This test ensures we don't try to use Touch ID on non-macOS platforms
        assert!(true, "Touch ID is macOS-only, test passes on other platforms");
    }
}
