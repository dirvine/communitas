// Debug keyring issue step by step
use keyring::Entry;
use uuid::Uuid;

#[tokio::main]
async fn main() {
    println!("=== Debug Keyring Step by Step ===");

    let service = "communitas-p2p";
    let test_user = format!("debug_user_{}", Uuid::new_v4());
    let key_name = format!("{}_master_key", test_user);

    println!("Service: {}", service);
    println!("Key name: {}", key_name);
    println!("Key name length: {}", key_name.len());

    // Step 1: Create entry and store
    println!("\n1. Creating entry and storing...");
    let entry1 = Entry::new(service, &key_name).expect("Failed to create entry1");
    entry1
        .set_password("test_value")
        .expect("Failed to set password");
    println!("✓ Successfully stored with entry1");

    // Step 2: Try to retrieve with the same entry instance
    println!("\n2. Retrieving with same entry instance...");
    match entry1.get_password() {
        Ok(password) => println!("✓ Retrieved with same instance: {}", password),
        Err(e) => println!("❌ Failed with same instance: {:?}", e),
    }

    // Step 3: Create new entry instance and try to retrieve immediately
    println!("\n3. Retrieving with new entry instance (immediate)...");
    let entry2 = Entry::new(service, &key_name).expect("Failed to create entry2");
    match entry2.get_password() {
        Ok(password) => println!("✓ Retrieved with new instance (immediate): {}", password),
        Err(e) => println!("❌ Failed with new instance (immediate): {:?}", e),
    }

    // Step 4: Small delay then try again
    println!("\n4. Waiting 100ms then retrieving with new entry instance...");
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
    let entry3 = Entry::new(service, &key_name).expect("Failed to create entry3");
    match entry3.get_password() {
        Ok(password) => println!("✓ Retrieved with new instance (after delay): {}", password),
        Err(e) => println!("❌ Failed with new instance (after delay): {:?}", e),
    }

    // Step 5: Longer delay
    println!("\n5. Waiting 500ms then retrieving with new entry instance...");
    tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
    let entry4 = Entry::new(service, &key_name).expect("Failed to create entry4");
    match entry4.get_password() {
        Ok(password) => println!(
            "✓ Retrieved with new instance (after longer delay): {}",
            password
        ),
        Err(e) => println!("❌ Failed with new instance (after longer delay): {:?}", e),
    }

    // Step 6: Try with different service/account combination
    println!("\n6. Trying with slightly different service name...");
    let alt_service = "communitas_p2p"; // underscore instead of dash
    let entry5 = Entry::new(alt_service, &key_name).expect("Failed to create entry5");
    match entry5.get_password() {
        Ok(password) => println!("✓ Retrieved with alt service: {}", password),
        Err(e) => println!("❌ Failed with alt service: {:?}", e),
    }

    // Cleanup
    println!("\n7. Cleaning up...");
    let _ = entry1.delete_credential();
    let _ = entry5.delete_credential(); // Try both service names

    println!("=== Debug Complete ===");
}
