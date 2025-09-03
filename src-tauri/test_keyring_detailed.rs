// Detailed keyring test to understand the issue
use keyring::Entry;

fn main() {
    println!("=== Detailed Keyring Test ===");

    let service = "communitas-p2p";
    let long_username = "test_user_23a1eb61-6274-4383-bdd6-a4daf11dd0e7_master_key";

    println!("Service: {}", service);
    println!("Username: {}", long_username);
    println!("Username length: {}", long_username.len());

    // Try to create an entry with the long name
    match Entry::new(service, long_username) {
        Ok(entry) => {
            println!("✓ Created keyring entry successfully");

            // Try to set a password
            match entry.set_password("test-password") {
                Ok(_) => {
                    println!("✓ Set password successfully");

                    // Immediately try to get the password back
                    match entry.get_password() {
                        Ok(password) => {
                            println!("✓ Retrieved password: {}", password);
                        }
                        Err(e) => {
                            println!("❌ Failed to get password: {}", e);
                        }
                    }

                    // Clean up
                    match entry.delete_credential() {
                        Ok(_) => println!("✓ Deleted credential successfully"),
                        Err(e) => println!("⚠ Failed to delete credential: {}", e),
                    }
                }
                Err(e) => {
                    println!("❌ Failed to set password: {}", e);
                }
            }
        }
        Err(e) => {
            println!("❌ Failed to create keyring entry: {}", e);
        }
    }

    println!("\n=== Testing with shorter name ===");
    let short_username = "test_user_master_key";
    println!("Short username: {}", short_username);
    println!("Short username length: {}", short_username.len());

    match Entry::new(service, short_username) {
        Ok(entry) => {
            println!("✓ Created keyring entry successfully");

            // Try to set a password
            match entry.set_password("test-password") {
                Ok(_) => {
                    println!("✓ Set password successfully");

                    // Immediately try to get the password back
                    match entry.get_password() {
                        Ok(password) => {
                            println!("✓ Retrieved password: {}", password);
                        }
                        Err(e) => {
                            println!("❌ Failed to get password: {}", e);
                        }
                    }

                    // Clean up
                    match entry.delete_credential() {
                        Ok(_) => println!("✓ Deleted credential successfully"),
                        Err(e) => println!("⚠ Failed to delete credential: {}", e),
                    }
                }
                Err(e) => {
                    println!("❌ Failed to set password: {}", e);
                }
            }
        }
        Err(e) => {
            println!("❌ Failed to create keyring entry: {}", e);
        }
    }
}
