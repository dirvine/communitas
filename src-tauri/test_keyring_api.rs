use keyring::Entry;

fn main() {
    let entry = Entry::new("test", "test").unwrap();
    // Try to see what methods are available
    // The correct method should be delete_password() not delete_credential()
    println!("Entry created");
}
