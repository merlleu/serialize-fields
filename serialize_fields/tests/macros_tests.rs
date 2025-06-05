use serialize_fields::{SerializeFields, SerializeFieldsTrait, contains, copy_selected_fields, filter_field_set};
use serde::{Serialize, Deserialize};
use std::collections::BTreeSet;

#[derive(SerializeFields, Serialize, Deserialize)]
struct TestUser {
    id: u32,
    name: String,
    email: Option<String>,
    profile: TestProfile,
}

#[derive(SerializeFields, Serialize, Deserialize, Clone)]
struct TestProfile {
    bio: String,
    avatar_url: Option<String>,
    social_links: Vec<String>,
}

fn create_test_user() -> TestUser {
    TestUser {
        id: 123,
        name: "Alice".to_string(),
        email: Some("alice@example.com".to_string()),
        profile: TestProfile {
            bio: "Software Engineer".to_string(),
            avatar_url: Some("https://example.com/avatar.jpg".to_string()),
            social_links: vec!["https://github.com/alice".to_string()],
        },
    }
}

#[test]
fn test_contains_macro() {
    let user = create_test_user();
    let mut selector = user.serialize_fields();
    
    selector.enable_dot_hierarchy("id");
    selector.enable_dot_hierarchy("profile.bio");

    // Test simple fields
    assert!(contains!(selector, id));
    assert!(!contains!(selector, name));
    assert!(!contains!(selector, email));

    // Test nested fields
    assert!(contains!(selector, profile.bio));
    assert!(!contains!(selector, profile.avatar_url));
}

#[test]
fn test_copy_selected_fields_macro() {
    let user = create_test_user();
    let mut selector = user.serialize_fields();
    
    selector.enable_dot_hierarchy("id");
    selector.enable_dot_hierarchy("name");
    // Note: email is not enabled

    #[derive(Debug, PartialEq)]
    struct PartialUser {
        id: Option<u32>,
        name: Option<String>,
        email: Option<String>,
    }

    let partial = copy_selected_fields!(selector, PartialUser {
        id: Some(user.id),
        name: Some(user.name.clone()),
        email: user.email.clone(),
    });

    assert_eq!(partial.id, Some(123));
    assert_eq!(partial.name, Some("Alice".to_string()));
    assert_eq!(partial.email, None); // Not enabled, so None
}

#[test]
fn test_filter_field_set_macro() {
    let user = create_test_user();
    let mut selector = user.serialize_fields();
    
    selector.enable_dot_hierarchy("id");
    selector.enable_dot_hierarchy("name");
    selector.enable_dot_hierarchy("profile.bio");
    // Note: email and profile.avatar_url are not enabled

    let field_set = filter_field_set!(selector, {
        id => format!("user_id"),
        name => format!("user_name"),
        email => format!("user_email"),
        profile.bio => format!("profile_biography"),
        profile.avatar_url => format!("profile_avatar"),
    });

    let expected: BTreeSet<String> = [
        "user_id".to_string(),
        "user_name".to_string(),
        "profile_biography".to_string(),
    ].into_iter().collect();

    assert_eq!(field_set, expected);
}

#[test]
fn test_combined_usage() {
    let user = create_test_user();
    let mut selector = user.serialize_fields();
    
    selector.enable_dot_hierarchy("id");
    selector.enable_dot_hierarchy("name");
    // Note: email and profile.bio are not enabled

    // Use contains! to check fields
    assert!(contains!(selector, id));
    assert!(contains!(selector, name));
    assert!(!contains!(selector, email));
    assert!(!contains!(selector, profile.bio));

    // Use copy_selected_fields! to create a partial struct
    // Field names must match the selector structure
    #[derive(Debug)]
    struct ApiResponse {
        id: Option<u32>,
        name: Option<String>,
        email: Option<String>,
    }

    let response = copy_selected_fields!(selector, ApiResponse {
        id: Some(user.id),
        name: Some(user.name.clone()),
        email: user.email.clone(),
    });

    assert_eq!(response.id, Some(123));
    assert_eq!(response.name, Some("Alice".to_string()));
    assert_eq!(response.email, None); // Not enabled

    // Use filter_field_set! to get enabled field names
    let enabled_fields = filter_field_set!(selector, {
        id => "id".to_string(),
        name => "name".to_string(),
        email => "email".to_string(),
        profile.bio => "profile.bio".to_string(),
    });

    assert_eq!(enabled_fields.len(), 2);
    assert!(enabled_fields.contains("id"));
    assert!(enabled_fields.contains("name"));
    assert!(!enabled_fields.contains("email"));
    assert!(!enabled_fields.contains("profile.bio"));
}

#[test]
fn test_empty_selector() {
    let user = create_test_user();
    let selector = user.serialize_fields(); // No fields enabled

    // contains! should return false for all fields
    assert!(!contains!(selector, id));
    assert!(!contains!(selector, name));
    assert!(!contains!(selector, profile.bio));

    // filter_field_set! should return empty set
    let field_set = filter_field_set!(selector, {
        id => "id".to_string(),
        name => "name".to_string(),
    });

    assert!(field_set.is_empty());
}

#[test]
fn test_complex_field_paths() {
    let user = create_test_user();
    let mut selector = user.serialize_fields();
    
    selector.enable_dot_hierarchy("profile.social_links");

    assert!(contains!(selector, profile.social_links));
    assert!(!contains!(selector, profile.bio));

    let field_set = filter_field_set!(selector, {
        profile.bio => "bio".to_string(),
        profile.social_links => "social".to_string(),
        profile.avatar_url => "avatar".to_string(),
    });

    assert_eq!(field_set.len(), 1);
    assert!(field_set.contains("social"));
}

// Alternative test showing manual field mapping when field names don't match selector structure
#[test]
fn test_manual_field_mapping() {
    let user = create_test_user();
    let mut selector = user.serialize_fields();
    
    selector.enable_dot_hierarchy("id");
    selector.enable_dot_hierarchy("profile.bio");
    // name and email are not enabled

    // Manual mapping approach for custom field names
    #[derive(Debug, PartialEq)]
    struct CustomApiResponse {
        user_id: Option<u32>,
        user_name: Option<String>,
        biography: Option<String>,
    }

    let custom_response = CustomApiResponse {
        user_id: if contains!(selector, id) { Some(user.id) } else { None },
        user_name: if contains!(selector, name) { Some(user.name.clone()) } else { None },
        biography: if contains!(selector, profile.bio) { Some(user.profile.bio.clone()) } else { None },
    };

    assert_eq!(custom_response.user_id, Some(123));
    assert_eq!(custom_response.user_name, None); // name not enabled
    assert_eq!(custom_response.biography, Some("Software Engineer".to_string())); // profile.bio enabled
}

#[test]
fn test_filter_field_set_multiple_outputs() {
    let user = create_test_user();
    let mut selector = user.serialize_fields();
    
    selector.enable_dot_hierarchy("id");
    selector.enable_dot_hierarchy("name");
    selector.enable_dot_hierarchy("profile.bio");
    // Note: email and profile.avatar_url are not enabled

    // Test multiple outputs per field using | separator
    let multi_field_set = filter_field_set!(selector, {
        id => "user_id".to_string() ; "id".to_string() ; "primary_key".to_string(),
        name => "user_name".to_string() ; "username".to_string(),
        email => "user_email".to_string() ; "email_address".to_string(),
        profile.bio => "bio".to_string() ; "biography".to_string() ; "profile_bio".to_string(),
        profile.avatar_url => "avatar".to_string() ; "profile_avatar".to_string(),
    });

    // All outputs for enabled fields should be present
    assert!(multi_field_set.contains("user_id"));
    assert!(multi_field_set.contains("id"));
    assert!(multi_field_set.contains("primary_key"));
    
    assert!(multi_field_set.contains("user_name"));
    assert!(multi_field_set.contains("username"));
    
    assert!(multi_field_set.contains("bio"));
    assert!(multi_field_set.contains("biography"));
    assert!(multi_field_set.contains("profile_bio"));
    
    // None of the outputs for disabled fields should be present
    assert!(!multi_field_set.contains("user_email"));
    assert!(!multi_field_set.contains("email_address"));
    assert!(!multi_field_set.contains("avatar"));
    assert!(!multi_field_set.contains("profile_avatar"));

    // Test that we have the correct total count
    assert_eq!(multi_field_set.len(), 8); // 3 for id + 2 for name + 3 for profile.bio
}