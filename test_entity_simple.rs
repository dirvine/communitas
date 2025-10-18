// Simple test to check for stack overflow in entity service
use communitas_core::crdt::EntityType;

#[test]
fn test_entity_type_as_str() {
    // Test that our EntityType as_str method works without stack overflow
    let group = EntityType::Group;
    assert_eq!(group.as_str(), "group");

    let channel = EntityType::Channel;
    assert_eq!(channel.as_str(), "channel");

    let project = EntityType::Project;
    assert_eq!(project.as_str(), "project");

    let org = EntityType::Organisation;
    assert_eq!(org.as_str(), "organisation");

    println!("✅ EntityType as_str test passed - no stack overflow");
}

#[test]
fn test_device_type_parse() {
    // Test that our DeviceType parse method works
    use communitas_core::types::DeviceType;

    assert_eq!(DeviceType::parse("desktop"), DeviceType::Desktop);
    assert_eq!(DeviceType::parse("unknown"), DeviceType::Unknown);

    println!("✅ DeviceType parse test passed - no stack overflow");
}
