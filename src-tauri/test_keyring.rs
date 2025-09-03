// Simple keyring test to verify macOS Keychain functionality
use keyring::Entry;

fn main() {
    println!("Testing keyring functionality on macOS...");

    let service = "test-communitas";
    let username = "test-user";

    // Try to create an entry
    match Entry::new(service, username) {
        Ok(entry) => {
            println!("✓ Created keyring entry successfully");

            // Try to set a password
            match entry.set_password("test-password") {
                Ok(_) => {
                    println!("✓ Set password successfully");

                    // Try to get the password back
                    match entry.get_password() {
                        Ok(password) => {
                            println!("✓ Retrieved password: {}", password);

                            // Clean up
                            match entry.delete_credential() {
                                Ok(_) => println!("✓ Deleted credential successfully"),
                                Err(e) => println!("⚠ Failed to delete credential: {}", e),
                            }
                        }
                        Err(e) => {
                            println!("✗ Failed to get password: {}", e);
                        }
                    }
                }
                Err(e) => {
                    println!("✗ Failed to set password: {}", e);
                }
            }
        }
        Err(e) => {
            println!("✗ Failed to create keyring entry: {}", e);
        }
    }
}
