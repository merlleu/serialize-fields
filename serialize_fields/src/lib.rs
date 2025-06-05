//! # SerializeFields
//!
//! A Rust procedural macro that enables **dynamic field selection** for struct serialization.
//! Control exactly which fields get serialized at runtime using a hierarchical field selector system.
//!
//! ## Features
//!
//! - 🎯 **Dynamic Field Selection**: Choose which fields to serialize at runtime
//! - 🌳 **Hierarchical Selection**: Use dot notation for nested structs (`"user.profile.name"`)
//! - 🔧 **Type Safe**: Compile-time validation of field paths
//! - 🚀 **Zero Runtime Cost**: Only enabled fields are processed during serialization
//! - 📦 **Serde Integration**: Works seamlessly with the serde ecosystem
//! - 🔄 **Collection Support**: Handles `Vec`, `Option`, `HashMap`, and other containers
//! - 🏗️ **Generic Architecture**: Single trait-based serialization implementation
//!
//! ## Quick Start
//!
//! ```rust
//! use serialize_fields::{SerializeFields, SerializeFieldsTrait};
//! use serde::{Serialize, Deserialize};
//!
//! #[derive(SerializeFields, Serialize, Deserialize)]
//! struct User {
//!     id: u32,
//!     name: Option<String>,
//!     email: Option<String>,
//! }
//!
//! let user = User {
//!     id: 123,
//!     name: Some("Alice".to_string()),
//!     email: Some("alice@example.com".to_string()),
//! };
//!
//! // Create field selector using the trait method
//! let mut fields = user.serialize_fields();
//! fields.enable_dot_hierarchy("id");
//! fields.enable_dot_hierarchy("name");
//!
//! // Serialize with selected fields only
//! let json = serde_json::to_string(&SerializeFields(&user, &fields)).unwrap();
//! // Output: {"id":123,"name":"Alice"}
//! ```
//!
//! ## Advanced Usage
//!
//! ### Nested Structs
//!
//! ```rust
//! # use serialize_fields::{SerializeFields, SerializeFieldsTrait};
//! # use serde::{Serialize, Deserialize};
//! #[derive(SerializeFields, Serialize, Deserialize)]
//! struct User {
//!     id: u32,
//!     profile: UserProfile,
//! }
//!
//! #[derive(SerializeFields, Serialize, Deserialize)]
//! struct UserProfile {
//!     bio: Option<String>,
//!     avatar_url: Option<String>,
//! }
//!
//! # let user = User {
//! #     id: 123,
//! #     profile: UserProfile {
//! #         bio: Some("Software Engineer".to_string()),
//! #         avatar_url: Some("https://example.com/avatar.jpg".to_string()),
//! #     },
//! # };
//! let mut fields = user.serialize_fields();
//! fields.enable_dot_hierarchy("id");
//! fields.enable_dot_hierarchy("profile.bio");  // Nested field selection
//!
//! let json = serde_json::to_string(&SerializeFields(&user, &fields)).unwrap();
//! // Output: {"id":123,"profile":{"bio":"Software Engineer"}}
//! ```
//!
//! ### Dynamic Field Selection
//!
//! ```rust
//! # use serialize_fields::{SerializeFields, SerializeFieldsTrait};
//! # use serde::{Serialize, Deserialize};
//! # #[derive(SerializeFields, Serialize, Deserialize)]
//! # struct User { id: u32, name: Option<String>, email: Option<String> }
//! # let user = User { id: 123, name: Some("Alice".to_string()), email: Some("alice@example.com".to_string()) };
//! fn serialize_user_with_fields(user: &User, requested_fields: &[&str]) -> String {
//!     let mut selector = user.serialize_fields();
//!     
//!     for field in requested_fields {
//!         selector.enable_dot_hierarchy(field);
//!     }
//!     
//!     serde_json::to_string(&SerializeFields(user, &selector)).unwrap()
//! }
//!
//! // Usage: GET /users/123?fields=id,name
//! let fields = vec!["id", "name"];
//! let json = serialize_user_with_fields(&user, &fields);
//! ```

#![doc(html_root_url = "https://docs.rs/serialize_fields/0.2.3")]
#![cfg_attr(docsrs, feature(doc_cfg))]

// Re-export the derive macro
pub use serialize_fields_macro::SerializeFields;

mod macros;

/// Trait for types that can provide field selectors for dynamic serialization.
///
/// This trait is automatically implemented by the `#[derive(SerializeFields)]` macro
/// and provides both field selector creation and serialization functionality.
///
/// # Examples
///
/// ```rust
/// # use serialize_fields::{SerializeFields as SerializeFieldsDerive, SerializeFieldsTrait};
/// # use serde::{Serialize, Deserialize};
/// #[derive(SerializeFieldsDerive, Serialize, Deserialize)]
/// struct User {
///     id: u32,
///     name: String,
/// }
///
/// # let user = User { id: 1, name: "Alice".to_string() };
/// // Create a field selector using the trait method
/// let mut fields = user.serialize_fields();
/// fields.enable_dot_hierarchy("id");
/// fields.enable_dot_hierarchy("name");
/// ```
pub trait SerializeFieldsTrait {
    /// The type of field selector for this struct.
    type FieldSelector: FieldSelector;

    /// Create a new field selector for this type.
    ///
    /// This is a convenience method that's equivalent to calling
    /// `{StructName}SerializeFieldSelector::new()` but provides a more
    /// ergonomic API through the trait.
    fn serialize_fields(&self) -> Self::FieldSelector;

    /// Serialize this struct using the provided field selector.
    ///
    /// This method is called by the generic `Serialize` implementation
    /// for `SerializeFields` and handles the actual serialization logic
    /// with field filtering.
    ///
    /// # Arguments
    ///
    /// * `field_selector` - The field selector that determines which fields to include
    /// * `serializer` - The serde serializer to use
    fn serialize<__S>(
        &self,
        field_selector: &Self::FieldSelector,
        __serializer: __S,
    ) -> Result<__S::Ok, __S::Error>
    where
        __S: serde::Serializer;
}

/// A wrapper struct that combines data with a field selector for serialization.
///
/// This is the core type that enables dynamic field selection. It wraps your data
/// and a field selector, implementing `Serialize` to only include enabled fields.
///
/// # Type Parameters
///
/// - `T`: The type of data being serialized
/// - `S`: The type of field selector (typically generated by the derive macro)
///
/// # Examples
///
/// ```rust
/// # use serialize_fields::{SerializeFields, SerializeFields as SerializeFieldsDerive};
/// # use serde::{Serialize, Deserialize};
/// # #[derive(SerializeFieldsDerive, Serialize, Deserialize)]
/// # struct User { id: u32, name: String }
/// # let user = User { id: 1, name: "Alice".to_string() };
/// # let mut selector = UserSerializeFieldSelector::new();
/// # selector.enable_dot_hierarchy("id");
/// let wrapper = SerializeFields(&user, &selector);
/// let json = serde_json::to_string(&wrapper).unwrap();
/// ```
pub struct SerializeFields<'a, T, S>(pub &'a T, pub &'a S);

impl<'a, T, S> serde::Serialize for SerializeFields<'a, T, S>
where
    T: SerializeFieldsTrait<FieldSelector = S>,
    S: FieldSelector,
{
    fn serialize<Se>(&self, serializer: Se) -> Result<Se::Ok, Se::Error>
    where
        Se: serde::Serializer,
    {
        self.0.serialize(self.1, serializer)
    }
}

// Generic implementation for Vec<T> where T implements SerializeFieldsTrait
impl<'a, T, S> serde::Serialize for SerializeFields<'a, Vec<T>, S>
where
    T: SerializeFieldsTrait<FieldSelector = S>,
    S: FieldSelector,
{
    fn serialize<Se>(&self, serializer: Se) -> Result<Se::Ok, Se::Error>
    where
        Se: serde::Serializer,
    {
        use serde::ser::SerializeSeq;

        let data = self.0;
        let field_selector = self.1;

        let mut seq = serializer.serialize_seq(Some(data.len()))?;

        for item in data {
            seq.serialize_element(&SerializeFields(item, field_selector))?;
        }

        seq.end()
    }
}

// Generic implementation for Option<T> where T implements SerializeFieldsTrait
impl<'a, T, S> serde::Serialize for SerializeFields<'a, Option<T>, S>
where
    T: SerializeFieldsTrait<FieldSelector = S>,
    S: FieldSelector,
{
    fn serialize<Se>(&self, serializer: Se) -> Result<Se::Ok, Se::Error>
    where
        Se: serde::Serializer,
    {
        let data = self.0;
        let field_selector = self.1;

        match data {
            Some(inner) => SerializeFields(inner, field_selector).serialize(serializer),
            None => serializer.serialize_none(),
        }
    }
}

// implement JsonSchema for SerializeFields<T, S> where T implements JsonSchema
#[cfg(feature = "schemars")]
impl<'a, T, S> schemars::JsonSchema for SerializeFields<'a, T, S>
where
    T: schemars::JsonSchema,
    S: FieldSelector,
{
    // Required methods
    fn schema_name() -> std::borrow::Cow<'static, str> {
        T::schema_name()
    }
    fn json_schema(generator: &mut schemars::SchemaGenerator) -> schemars::Schema {
        T::json_schema(generator)
    }

    // Provided methods
    fn always_inline_schema() -> bool {
        T::always_inline_schema()
    }
    fn inline_schema() -> bool {
        T::inline_schema()
    }
    fn schema_id() -> std::borrow::Cow<'static, str> {
        T::schema_id()
    }
}

/// Helper trait for field selectors to provide common functionality.
///
/// This trait is automatically implemented for all generated field selectors.
pub trait FieldSelector {
    /// Create a new selector with all fields disabled.
    fn new() -> Self;

    /// Enable a field using dot notation.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// selector.enable_dot_hierarchy("name");           // Simple field
    /// selector.enable_dot_hierarchy("profile.bio");    // Nested field
    /// selector.enable_dot_hierarchy("posts.title");    // Field in collection
    /// ```
    fn enable_dot_hierarchy(&mut self, field: &str);

    /// Enable a field using a slice of field names.
    ///
    /// This is useful when you already have the field path split.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// selector.enable(&["name"]);                  // Simple field
    /// selector.enable(&["profile", "bio"]);       // Nested field
    /// ```
    fn enable(&mut self, field_hierarchy: &[&str]);
}

/// Utility functions for working with field selectors.
pub mod utils {
    /// Parse a comma-separated list of field names.
    ///
    /// This is useful for parsing query parameters or configuration strings.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use serialize_fields::utils::parse_field_list;
    ///
    /// let fields = parse_field_list("id,name,profile.bio");
    /// assert_eq!(fields, vec!["id", "name", "profile.bio"]);
    /// ```
    pub fn parse_field_list(fields: &str) -> Vec<&str> {
        fields
            .split(',')
            .map(|s| s.trim())
            .filter(|s| !s.is_empty())
            .collect()
    }

    /// Create a field selector from a list of field names.
    ///
    /// This is a convenience function that combines parsing and enabling fields.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use serialize_fields::utils::create_selector_from_list;
    ///
    /// let selector: UserSerializeFieldSelector =
    ///     create_selector_from_list("id,name,profile.bio");
    /// ```
    pub fn create_selector_from_list<T>(fields: &str) -> T
    where
        T: crate::FieldSelector,
    {
        let mut selector = T::new();
        for field in parse_field_list(fields) {
            selector.enable_dot_hierarchy(field);
        }
        selector
    }
}
