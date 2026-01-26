# SerializeFields

A Rust procedural macro that enables **dynamic field selection** for struct serialization. Control exactly which fields get serialized at runtime using a hierarchical field selector system.

## Features

- **Dynamic Field Selection**: Choose which fields to serialize at runtime
- **Hierarchical Selection**: Use dot notation for nested structs (`"user.profile.name"`)
- **Type Safe**: Compile-time validation of field paths
- **Zero Runtime Cost**: Only enabled fields are processed during serialization
- **Serde Integration**: Works seamlessly with the serde ecosystem
- **Collection Support**: Handles `Vec`, `Option`, `HashMap`, and other containers

## Quick Start

Add to your `Cargo.toml`:

```toml
[dependencies]
serialize_fields = "0.1.0"
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
```

## Basic Usage

```rust
use serialize_fields::{SerializeFields, SerializeFieldsTrait};
#[derive(SerializeFields)]
struct User {
    id: u32,
    name: Option<String>,
    email: Option<String>,
    profile: UserProfile,
}

#[derive(SerializeFields)]
struct UserProfile {
    bio: Option<String>,
    avatar_url: Option<String>,
    social_links: Vec<String>,
}

fn main() {
    let user = User {
        id: 123,
        name: Some("Alice".to_string()),
        email: Some("alice@example.com".to_string()),
        profile: UserProfile {
            bio: Some("Software Engineer".to_string()),
            avatar_url: Some("https://example.com/avatar.jpg".to_string()),
            social_links: vec!["https://github.com/alice".to_string()],
        },
    };

    // Create field selector using the trait method
    let mut fields = user.serialize_fields();
    fields.enable_dot_hierarchy("id");
    fields.enable_dot_hierarchy("name");
    fields.enable_dot_hierarchy("profile.bio");

    // Serialize with selected fields only
    let json = serde_json::to_string_pretty(&SerializeFields(&user, &fields)).unwrap();
    println!("{}", json);
}
```

**Output:**
```json
{
  "id": 123,
  "name": "Alice",
  "profile": {
    "bio": "Software Engineer"
  }
}
```

## Advanced Examples

### API Response Filtering

Perfect for REST APIs where different endpoints need different field subsets:

```rust
use serialize_fields::{SerializeFields, SerializeFieldsTrait};

// Public profile view - minimal fields
fn get_public_profile(user: &User) -> String {
    let mut fields = user.serialize_fields();
    fields.enable_dot_hierarchy("id");
    fields.enable_dot_hierarchy("name");
    fields.enable_dot_hierarchy("profile.bio");
    
    serde_json::to_string(&SerializeFields(user, &fields)).unwrap()
}

// Admin view - all fields
fn get_admin_profile(user: &User) -> String {
    let mut fields = user.serialize_fields();
    fields.enable_dot_hierarchy("id");
    fields.enable_dot_hierarchy("name");
    fields.enable_dot_hierarchy("email");
    fields.enable_dot_hierarchy("profile.bio");
    fields.enable_dot_hierarchy("profile.avatar_url");
    fields.enable_dot_hierarchy("profile.social_links");
    
    serde_json::to_string(&SerializeFields(user, &fields)).unwrap()
}
```

### Dynamic Field Selection from Query Parameters

```rust
use std::collections::HashSet;

fn serialize_user_with_fields(user: &User, requested_fields: &[&str]) -> String {
    let mut selector = user.serialize_fields();
    
    for field in requested_fields {
        selector.enable_dot_hierarchy(field);
    }
    
    serde_json::to_string(&SerializeFields(user, &selector)).unwrap()
}

// Usage: GET /users/123?fields=id,name,profile.bio
let fields = vec!["id", "name", "profile.bio"];
let json = serialize_user_with_fields(&user, &fields);
```

### Conditional Field Inclusion

```rust
fn serialize_user_for_role(user: &User, role: &str) -> String {
    let mut fields = user.serialize_fields();
    
    // Always include basic fields
    fields.enable_dot_hierarchy("id");
    fields.enable_dot_hierarchy("name");
    
    match role {
        "admin" => {
            fields.enable_dot_hierarchy("email");
            fields.enable_dot_hierarchy("profile.bio");
            fields.enable_dot_hierarchy("profile.avatar_url");
            fields.enable_dot_hierarchy("profile.social_links");
        }
        "user" => {
            fields.enable_dot_hierarchy("profile.bio");
            fields.enable_dot_hierarchy("profile.avatar_url");
        }
        "public" => {
            fields.enable_dot_hierarchy("profile.bio");
        }
        _ => {}
    }
    
    serde_json::to_string(&SerializeFields(user, &fields)).unwrap()
}
```
