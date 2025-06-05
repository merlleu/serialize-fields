//! Utility macros for working with field selectors
//!
//! This module provides convenient macros for common operations with field selectors:
//! - `contains!`: Check if a field path is enabled
//! - `copy_selected_fields!`: Create structs with conditional field copying
//! - `filter_field_set!`: Create filtered sets of enabled field paths

/// Check if a field path is enabled in a field selector.
///
/// This macro uses non-recursive Option chaining for levels 1-2, and recursive
/// calls for deeper nesting (3+ levels) to balance performance and simplicity.
///
/// # Examples
///
/// ```rust
/// # use serialize_fields::{SerializeFields, SerializeFieldsTrait, contains};
/// # use serde::{Serialize, Deserialize};
/// # #[derive(SerializeFields, Serialize, Deserialize)]
/// # struct User { id: u32, name: String, profile: Profile }
/// # #[derive(SerializeFields, Serialize, Deserialize)]  
/// # struct Profile { bio: String }
/// # let user = User { id: 1, name: "Alice".to_string(), profile: Profile { bio: "Developer".to_string() } };
/// let mut selector = user.serialize_fields();
/// selector.enable_dot_hierarchy("id");
/// selector.enable_dot_hierarchy("profile.bio");
///
/// assert!(contains!(selector, id));
/// assert!(contains!(selector, profile.bio));
/// assert!(!contains!(selector, name));
/// ```
#[macro_export]
macro_rules! contains {
    // Level 1: Single field - direct check
    ($selector:expr, $field:ident) => {
        $selector.$field.is_some()
    };
    
    // Level 2: Two fields - Option chaining (non-recursive)
    ($selector:expr, $field1:ident . $field2:ident) => {
        $selector.$field1.as_ref().map_or(false, |nested| nested.$field2.is_some())
    };
    
    // Level 3+: Three or more fields - recursive for simplicity
    ($selector:expr, $field:ident . $($rest:ident).+) => {
        if let Some(ref nested) = $selector.$field {
            contains!(nested, $($rest).+)
        } else {
            false
        }
    };
}

/// Copy selected fields from a source struct to a new struct, using provided blocks for enabled fields.
///
/// For each field, if it's enabled in the field selector, the corresponding block is evaluated.
/// Otherwise, the field is set to `None` (for Option fields) or a default value.
///
/// # Examples
///
/// ```rust
/// # use serialize_fields::{SerializeFields, SerializeFieldsTrait, copy_selected_fields};
/// # use serde::{Serialize, Deserialize};
/// # #[derive(SerializeFields, Serialize, Deserialize)]
/// # struct User { id: u32, name: String, email: Option<String> }
/// # let user = User { id: 1, name: "Alice".to_string(), email: Some("alice@example.com".to_string()) };
/// let mut selector = user.serialize_fields();
/// selector.enable_dot_hierarchy("id");
/// selector.enable_dot_hierarchy("name");
///
/// #[derive(Debug)]
/// struct PartialUser {
///     id: Option<u32>,
///     name: Option<String>,
///     email: Option<String>,
/// }
///
/// let partial = copy_selected_fields!(selector, PartialUser {
///     id: Some(user.id),
///     name: Some(user.name.clone()),
///     email: user.email.clone()
/// });
///
/// assert_eq!(partial.id, Some(1));
/// assert_eq!(partial.name, Some("Alice".to_string()));
/// assert_eq!(partial.email, None); // Not enabled in selector
/// ```
#[macro_export]
macro_rules! copy_selected_fields {
    // Main entry point: copy_selected_fields!(selector, StructName { field1: block1, field2: block2, ... })
    ($selector:expr, $struct_name:ident { $($field:ident: $block:expr),* $(,)? }) => {
        $struct_name {
            $(
                $field: if $crate::contains!($selector, $field) {
                    $block
                } else {
                    None
                },
            )*
        }
    };
}

/// Create a BTreeSet of enabled field paths that match specified patterns.
///
/// This macro checks which fields are enabled and formats them according to the provided patterns.
/// It supports multiple outputs per field using the `|` separator for cases where one field
/// should map to multiple output values. Field paths use dot notation for nested access.
///
/// # Examples
///
/// ```rust
/// # use serialize_fields::{SerializeFields, SerializeFieldsTrait, filter_field_set};
/// # use serde::{Serialize, Deserialize};
/// # #[derive(SerializeFields, Serialize, Deserialize)]
/// # struct User { id: u32, name: String, profile: Profile }
/// # #[derive(SerializeFields, Serialize, Deserialize)]
/// # struct Profile { bio: String, avatar: String }
/// # let user = User { 
/// #     id: 1, 
/// #     name: "Alice".to_string(), 
/// #     profile: Profile { bio: "Developer".to_string(), avatar: "avatar.jpg".to_string() }
/// # };
/// let mut selector = user.serialize_fields();
/// selector.enable_dot_hierarchy("id");
/// selector.enable_dot_hierarchy("name");
/// selector.enable_dot_hierarchy("profile.bio");
///
/// // Single output per field
/// let field_set = filter_field_set!(selector, {
///     id => format!("user_id"),
///     name => format!("user_name"),
///     profile.bio => format!("profile_bio"),
///     profile.avatar => format!("profile_avatar")
/// });
///
/// // Multiple outputs per field using ; separator
/// let multi_field_set = filter_field_set!(selector, {
///     id => "user_id".to_string() ; "id".to_string() ; "primary_key".to_string(),
///     name => "user_name".to_string() ; "username".to_string(),
///     profile.bio => "bio".to_string() ; "biography".to_string() ; "profile_bio".to_string(),
/// });
///
/// // Only enabled fields are included (all outputs for that field)
/// assert!(multi_field_set.contains("user_id"));
/// assert!(multi_field_set.contains("id"));
/// assert!(multi_field_set.contains("primary_key"));
/// assert!(multi_field_set.contains("bio"));
/// assert!(multi_field_set.contains("biography"));
/// assert!(multi_field_set.contains("profile_bio"));
/// ```
#[macro_export]
macro_rules! filter_field_set {
    // Main pattern: handles dot-separated field paths with multiple outputs
    ($selector:expr, { 
        $(
            $($field_part:tt).+ => $($output:expr);+
        ),* $(,)?
    }) => {
        {
            let mut set = ::std::collections::BTreeSet::new();
            $(
                if $crate::filter_field_set_contains!($selector, $($field_part).+) {
                    $(
                        set.insert($output);
                    )+
                }
            )*
            set
        }
    };
}

/// Helper macro to check field paths of any depth for filter_field_set!
#[macro_export]
macro_rules! filter_field_set_contains {
    // Single field: field
    ($selector:expr, $field:tt) => {
        $crate::contains!($selector, $field)
    };
    
    // Nested fields: field.subfield.deeper.etc
    ($selector:expr, $field:tt . $($rest:tt).+) => {
        $crate::contains!($selector, $field . $($rest).+)
    };
}

/// Helper macro for filter_field_set! to handle different field path patterns
#[macro_export]
macro_rules! filter_field_set_helper {
    // Simple field
    ($selector:expr, $field:ident) => {
        $crate::contains!($selector, $field)
    };
    
    // Nested field with dots - reconstruct the path for contains!
    ($selector:expr, $field:ident . $($rest:ident).+) => {
        $crate::contains!($selector, $field . $($rest).+)
    };
}

/// Helper macro for creating field path patterns in filter_field_set!
/// 
/// This handles the conversion of dot notation to nested field access.
#[macro_export]
macro_rules! field_path {
    ($field:ident) => { $field };
    ($field:ident . $($rest:ident).+) => { $field . $($rest).+ };
}