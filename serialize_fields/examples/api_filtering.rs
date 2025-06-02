//! API filtering example for serialize_fields
//! 
//! This example demonstrates how to use serialize_fields in a REST API context
//! where different endpoints or user roles need different field subsets.

use serialize_fields::{SerializeFields, FieldSelector, SerializeFieldsTrait};
use serde::{Serialize, Deserialize};
use std::collections::HashMap;

#[derive(SerializeFields, Serialize, Deserialize, Debug, Clone)]
struct User {
    id: u32,
    username: String,
    email: String,
    password_hash: String,
    first_name: String,
    last_name: String,
    profile: UserProfile,
    settings: UserSettings,
    created_at: String,
    last_login: Option<String>,
}

#[derive(SerializeFields, Serialize, Deserialize, Debug, Clone)]
struct UserProfile {
    bio: Option<String>,
    avatar_url: Option<String>,
    website: Option<String>,
    location: Option<String>,
    social_links: Vec<SocialLink>,
    stats: ProfileStats,
}

#[derive(SerializeFields, Serialize, Deserialize, Debug, Clone)]
struct SocialLink {
    platform: String,
    username: String,
    url: String,
}

#[derive(SerializeFields, Serialize, Deserialize, Debug, Clone)]
struct ProfileStats {
    followers_count: u32,
    following_count: u32,
    posts_count: u32,
}

#[derive(SerializeFields, Serialize, Deserialize, Debug, Clone)]
struct UserSettings {
    theme: String,
    language: String,
    notifications_enabled: bool,
    privacy_level: String,
}

/// Simulates different API endpoint responses with appropriate field filtering
fn main() {
    let user = create_sample_user();

    println!("=== API Endpoint Field Filtering Examples ===\n");

    // Public profile view (what anyone can see)
    println!("1. GET /api/users/{}/profile (Public View):");
    let public_json = get_public_profile(&user);
    println!("{}\n", public_json);

    // User's own profile (what they see about themselves)
    println!("2. GET /api/users/me (Own Profile):");
    let own_profile_json = get_own_profile(&user);
    println!("{}\n", own_profile_json);

    // Admin view (what admins see)
    println!("3. GET /api/admin/users/{} (Admin View):");
    let admin_json = get_admin_view(&user);
    println!("{}\n", admin_json);

    // Search results (minimal info for lists)
    println!("4. GET /api/users/search (Search Results):");
    let search_json = get_search_result(&user);
    println!("{}\n", search_json);

    // API with field selection query parameter
    println!("5. GET /api/users/{}?fields=id,username,profile.bio,profile.stats (Custom Fields):");
    let custom_fields = vec!["id", "username", "profile.bio", "profile.stats.followers_count"];
    let custom_json = get_user_with_fields(&user, &custom_fields);
    println!("{}\n", custom_json);

    // Different user roles
    println!("6. Role-based access:");
    for role in &["guest", "user", "moderator", "admin"] {
        println!("   Role '{}': ", role);
        let role_json = get_user_for_role(&user, role);
        println!("{}", role_json);
    }
}

/// Public profile - only non-sensitive information
fn get_public_profile(user: &User) -> String {
    let mut fields = user.serialize_fields(); // Using the trait method
    fields.enable_dot_hierarchy("id");
    fields.enable_dot_hierarchy("username");
    fields.enable_dot_hierarchy("first_name");
    fields.enable_dot_hierarchy("last_name");
    fields.enable_dot_hierarchy("profile.bio");
    fields.enable_dot_hierarchy("profile.avatar_url");
    fields.enable_dot_hierarchy("profile.website");
    fields.enable_dot_hierarchy("profile.location");
    fields.enable_dot_hierarchy("profile.social_links");
    fields.enable_dot_hierarchy("profile.stats");
    
    serde_json::to_string_pretty(&SerializeFields(user, &fields)).unwrap()
}

/// User's own profile - includes private settings but not sensitive data
fn get_own_profile(user: &User) -> String {
    let mut fields = UserSerializeFieldSelector::new();
    fields.enable_dot_hierarchy("id");
    fields.enable_dot_hierarchy("username");
    fields.enable_dot_hierarchy("email");
    fields.enable_dot_hierarchy("first_name");
    fields.enable_dot_hierarchy("last_name");
    fields.enable_dot_hierarchy("profile");
    fields.enable_dot_hierarchy("settings");
    fields.enable_dot_hierarchy("created_at");
    fields.enable_dot_hierarchy("last_login");
    
    serde_json::to_string_pretty(&SerializeFields(user, &fields)).unwrap()
}

/// Admin view - includes everything except password hash
fn get_admin_view(user: &User) -> String {
    let mut fields = UserSerializeFieldSelector::new();
    fields.enable_dot_hierarchy("id");
    fields.enable_dot_hierarchy("username");
    fields.enable_dot_hierarchy("email");
    fields.enable_dot_hierarchy("first_name");
    fields.enable_dot_hierarchy("last_name");
    fields.enable_dot_hierarchy("profile");
    fields.enable_dot_hierarchy("settings");
    fields.enable_dot_hierarchy("created_at");
    fields.enable_dot_hierarchy("last_login");
    // Note: password_hash is intentionally excluded
    
    serde_json::to_string_pretty(&SerializeFields(user, &fields)).unwrap()
}

/// Search results - minimal information for performance
fn get_search_result(user: &User) -> String {
    let mut fields = UserSerializeFieldSelector::new();
    fields.enable_dot_hierarchy("id");
    fields.enable_dot_hierarchy("username");
    fields.enable_dot_hierarchy("first_name");
    fields.enable_dot_hierarchy("last_name");
    fields.enable_dot_hierarchy("profile.avatar_url");
    
    serde_json::to_string_pretty(&SerializeFields(user, &fields)).unwrap()
}

/// Custom field selection (e.g., from query parameters)
fn get_user_with_fields(user: &User, requested_fields: &[&str]) -> String {
    let mut fields = UserSerializeFieldSelector::new();
    
    for field in requested_fields {
        fields.enable_dot_hierarchy(field);
    }
    
    serde_json::to_string_pretty(&SerializeFields(user, &fields)).unwrap()
}

/// Role-based access control
fn get_user_for_role(user: &User, role: &str) -> String {
    let mut fields = UserSerializeFieldSelector::new();
    
    // Base fields for all roles
    fields.enable_dot_hierarchy("id");
    fields.enable_dot_hierarchy("username");
    
    match role {
        "guest" => {
            // Guests see very limited info
            fields.enable_dot_hierarchy("first_name");
        }
        "user" => {
            // Regular users see public profile info
            fields.enable_dot_hierarchy("first_name");
            fields.enable_dot_hierarchy("last_name");
            fields.enable_dot_hierarchy("profile.bio");
            fields.enable_dot_hierarchy("profile.avatar_url");
        }
        "moderator" => {
            // Moderators see more but not private settings
            fields.enable_dot_hierarchy("first_name");
            fields.enable_dot_hierarchy("last_name");
            fields.enable_dot_hierarchy("email");
            fields.enable_dot_hierarchy("profile");
            fields.enable_dot_hierarchy("created_at");
            fields.enable_dot_hierarchy("last_login");
        }
        "admin" => {
            // Admins see everything except password
            fields.enable_dot_hierarchy("email");
            fields.enable_dot_hierarchy("first_name");
            fields.enable_dot_hierarchy("last_name");
            fields.enable_dot_hierarchy("profile");
            fields.enable_dot_hierarchy("settings");
            fields.enable_dot_hierarchy("created_at");
            fields.enable_dot_hierarchy("last_login");
        }
        _ => {}
    }
    
    serde_json::to_string(&SerializeFields(user, &fields)).unwrap()
}

fn create_sample_user() -> User {
    User {
        id: 42,
        username: "alice_dev".to_string(),
        email: "alice@example.com".to_string(),
        password_hash: "".to_string(),
        first_name: "Alice".to_string(),
        last_name: "Johnson".to_string(),
        profile: UserProfile {
            bio: Some("Senior Software Engineer passionate about Rust and open source".to_string()),
            avatar_url: Some("https://example.com/avatars/alice.jpg".to_string()),
            website: Some("https://alicejohnson.dev".to_string()),
            location: Some("San Francisco, CA".to_string()),
            social_links: vec![
                SocialLink {
                    platform: "GitHub".to_string(),
                    username: "alice_dev".to_string(),
                    url: "https://github.com/alice_dev".to_string(),
                },
                SocialLink {
                    platform: "Twitter".to_string(),
                    username: "alice_codes".to_string(),
                    url: "https://twitter.com/alice_codes".to_string(),
                },
            ],
            stats: ProfileStats {
                followers_count: 1542,
                following_count: 234,
                posts_count: 87,
            },
        },
        settings: UserSettings {
            theme: "dark".to_string(),
            language: "en".to_string(),
            notifications_enabled: true,
            privacy_level: "public".to_string(),
        },
        created_at: "2023-01-15T10:30:00Z".to_string(),
        last_login: Some("2024-01-15T14:22:33Z".to_string()),
    }
}