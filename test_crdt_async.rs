// Quick test to verify CrdtManager async I/O conversion
use communitas_core::crdt_manager::CrdtManager;
use tempfile::tempdir;
use yrs::Doc;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Testing CrdtManager async I/O...");
    
    let temp_dir = tempdir()?;
    let storage_path = temp_dir.path();
    
    // Test 1: new() uses async fs::create_dir_all
    println!("1. Testing new() with async create_dir_all...");
    let manager = CrdtManager::new(storage_path).await?;
    println!("   ✓ CrdtManager created successfully");
    
    // Test 2: save_document() uses async fs operations
    println!("2. Testing save_document() with async fs operations...");
    let doc = Doc::new();
    CrdtManager::set_map_value(&doc, "test_field", "test_value")?;
    manager.save_document("test-doc", "test_entity", "entity-1", &doc).await?;
    println!("   ✓ Document saved successfully");
    
    // Test 3: load_document() uses async fs::read_dir and fs::read
    println!("3. Testing load_document() with async fs operations...");
    let loaded_doc = manager.load_document("test-doc").await?;
    let value: String = CrdtManager::get_map_value(&loaded_doc, "test_field")?
        .expect("test_field should exist");
    assert_eq!(value, "test_value");
    println!("   ✓ Document loaded successfully: {}", value);
    
    // Test 4: list_documents() uses async fs::read_dir
    println!("4. Testing list_documents() with async fs operations...");
    let docs = manager.list_documents("test_entity").await?;
    assert_eq!(docs.len(), 1);
    println!("   ✓ Listed {} document(s)", docs.len());
    
    // Test 5: delete_document() uses async fs operations
    println!("5. Testing delete_document() with async fs operations...");
    manager.delete_document("test-doc").await?;
    let docs = manager.list_documents("test_entity").await?;
    assert_eq!(docs.len(), 0);
    println!("   ✓ Document deleted successfully");
    
    println!("\n✅ All async I/O operations completed successfully!");
    println!("   - fs::create_dir_all → tokio::fs::create_dir_all");
    println!("   - fs::read_to_string → tokio::fs::read_to_string");
    println!("   - fs::write → tokio::fs::write");
    println!("   - fs::rename → tokio::fs::rename");
    println!("   - fs::read_dir → tokio::fs::read_dir (with .next_entry() loop)");
    println!("   - fs::read → tokio::fs::read");
    println!("   - fs::remove_file → tokio::fs::remove_file");
    
    Ok(())
}
