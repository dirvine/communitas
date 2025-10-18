// Test file for Yrs API compatibility - TDD approach
// This will help us verify the Yrs API migration works correctly

use yrs::{Doc, MapPrelim, Transact};

#[test]
fn test_yrs_api_compatibility() {
    // Test that we can create Yrs documents and maps correctly
    let doc = Doc::new();

    // Test the current API that should work
    {
        let mut txn = doc.transact_mut();
        let map = doc.get_or_insert_map("test");

        // Test inserting values
        map.insert(&mut txn, "key1", "value1");
        map.insert(&mut txn, "key2", 42i64);

        // Test reading values back
        let txn_read = doc.transact();
        assert_eq!(map.get(&txn_read, "key1"), Some(yrs::Out::from("value1")));
        assert_eq!(map.get(&txn_read, "key2"), Some(yrs::Out::from(42i64)));
    }

    println!("✅ Yrs API basic test passed");
}

#[test]
fn test_map_prelim_creation() {
    // Test creating MapPrelim correctly
    // This should help us fix the MapPrelim construction error

    // The error shows: struct takes 0 generic arguments but 1 generic argument was supplied
    // So MapPrelim should not have generic parameters
    let _map_prelim = MapPrelim::new();

    println!("✅ MapPrelim creation test passed");
}

#[test]
fn test_crdt_manager_api() {
    // Test that CrdtManager API works as expected
    // This will help us understand what methods are available

    // We'll need to create a real test with CrdtManager
    // For now, just verify the import works
    use communitas_core::CrdtManager;

    println!("✅ CrdtManager import test passed");
}
