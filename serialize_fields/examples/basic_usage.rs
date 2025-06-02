//! Basic usage example for serialize_fields
//! 
//! This example demonstrates the fundamental concepts:
//! - Deriving SerializeFields on structs
//! - Creating field selectors
//! - Enabling specific fields
//! - Serializing with field selection

use serialize_fields::{SerializeFields, SerializeFieldsTrait};
use serde::{Serialize, Deserialize};

#[derive(SerializeFields, Serialize, Deserialize, Debug)]
struct User {
    id: u32,
    name: String,
    email: Option<String>,
    phone: Option<String>,
    profile: UserProfile,
}

#[derive(SerializeFields, Serialize, Deserialize, Debug)]
struct UserProfile {
    bio: String,
    avatar_url: Option<String>,
    website: Option<String>,
    social_links: Vec<String>,
}

fn main() {
    // Create sample data
    let user = User {
        id: 123,
        name: "Alice Johnson".to_string(),
        email: Some("alice@example.com".to_string()),
        phone: Some("+1-555-0123".to_string()),
        profile: UserProfile {
            bio: "Software Engineer passionate about Rust".to_string(),
            avatar_url: Some("https://example.com/avatars/alice.jpg".to_string()),
            website: Some("https://alicejohnson.dev".to_string()),
            social_links: vec![
                "https://github.com/alice".to_string(),
                "https://twitter.com/alice_codes".to_string(),
            ],
        },
    };

    println!("=== Basic Usage Examples ===\n");

    // Example 1: Select only basic fields
    println!("1. Basic fields only (id, name):");
    let mut basic_fields = UserSerializeFieldSelector::new();
    basic_fields.enable_dot_hierarchy("id");
    basic_fields.enable_dot_hierarchy("name");
    
    let json = serde_json::to_string_pretty(&SerializeFields(&user, &basic_fields)).unwrap();
    println!("{}\n", json);

    // Example 2: Include contact information
    println!("2. Contact information (id, name, email, phone):");
    let mut contact_fields = UserSerializeFieldSelector::new();
    contact_fields.enable_dot_hierarchy("id");
    contact_fields.enable_dot_hierarchy("name");
    contact_fields.enable_dot_hierarchy("email");
    contact_fields.enable_dot_hierarchy("phone");
    
    let json = serde_json::to_string_pretty(&SerializeFields(&user, &contact_fields)).unwrap();
    println!("{}\n", json);

    // Example 3: Select nested profile fields
    println!("3. Profile information only:");
    let mut profile_fields = UserSerializeFieldSelector::new();
    profile_fields.enable_dot_hierarchy("id");
    profile_fields.enable_dot_hierarchy("name");
    profile_fields.enable_dot_hierarchy("profile.bio");
    profile_fields.enable_dot_hierarchy("profile.website");
    
    let json = serde_json::to_string_pretty(&SerializeFields(&user, &profile_fields)).unwrap();
    println!("{}\n", json);

    // Example 4: Social links only
    println!("4. Social links focus:");
    let mut social_fields = UserSerializeFieldSelector::new();
    social_fields.enable_dot_hierarchy("name");
    social_fields.enable_dot_hierarchy("profile.social_links");
    
    let json = serde_json::to_string_pretty(&SerializeFields(&user, &social_fields)).unwrap();
    println!("{}\n", json);

    // Example 5: Using the enable method with field hierarchy
    println!("5. Using enable() method with field hierarchy:");
    let mut hierarchy_fields = UserSerializeFieldSelector::new();
    hierarchy_fields.enable(&["id"]);
    hierarchy_fields.enable(&["email"]);
    hierarchy_fields.enable(&["profile", "avatar_url"]);
    
    let json = serde_json::to_string_pretty(&SerializeFields(&user, &hierarchy_fields)).unwrap();
    println!("{}\n", json);

    // Example 6: Everything (for comparison)
    println!("6. All fields (using regular serde):");
    let json = serde_json::to_string_pretty(&user).unwrap();
    println!("{}\n", json);

    // Example 7: Using the SerializeFieldsTrait
    println!("7. Using SerializeFieldsTrait:");
    let mut trait_fields = user.serialize_fields(); // Using the trait method
    trait_fields.enable_dot_hierarchy("id");
    trait_fields.enable_dot_hierarchy("name");
    trait_fields.enable_dot_hierarchy("profile.social_links");
    
    let json = serde_json::to_string_pretty(&SerializeFields(&user, &trait_fields)).unwrap();
    println!("{}\n", json);

    // Example 8: Using utility functions
    println!("8. Using utility functions:");
    let field_list = "id,name,profile.bio";
    let fields: UserSerializeFieldSelector = serialize_fields::utils::create_selector_from_list(field_list);
    
    let json = serde_json::to_string_pretty(&SerializeFields(&user, &fields)).unwrap();
    println!("Fields from list '{}': {}\n", field_list, json);

    // Example 9: Comparing trait method vs manual creation
    println!("9. Trait method vs manual creation comparison:");
    
    // Manual approach
    let mut manual_fields = UserSerializeFieldSelector::new();
    manual_fields.enable_dot_hierarchy("id");
    manual_fields.enable_dot_hierarchy("name");
    
    // Trait approach  
    let mut trait_fields2 = user.serialize_fields();
    trait_fields2.enable_dot_hierarchy("id");
    trait_fields2.enable_dot_hierarchy("name");
    
    let json_manual = serde_json::to_string(&SerializeFields(&user, &manual_fields)).unwrap();
    let json_trait = serde_json::to_string(&SerializeFields(&user, &trait_fields2)).unwrap();
    
    println!("   Manual approach: {}", json_manual);
    println!("   Trait approach:  {}", json_trait);
    println!("   Results identical: {}", json_manual == json_trait);
}