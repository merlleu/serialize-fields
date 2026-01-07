//! Integration tests for serialize_fields
//! 
//! These tests verify the macro works correctly with various struct configurations
//! and that the generated code functions as expected.

use serialize_fields::{SerializeFields, FieldSelector, SerializeFieldsTrait, contains, copy_selected_fields, filter_field_set};
use serde::{Serialize, Deserialize};
use serde_json::{Value, Map};

#[derive(SerializeFields, Serialize, Deserialize, Debug, PartialEq)]
struct SimpleStruct {
    id: u32,
    name: String,
    optional_field: Option<String>,
}

#[derive(SerializeFields, Serialize, Deserialize, Debug, PartialEq)]
struct NestedStruct {
    id: u32,
    inner: InnerStruct,
    optional_inner: Option<InnerStruct>,
}

#[derive(SerializeFields, Serialize, Deserialize, Debug, PartialEq)]
struct InnerStruct {
    value: String,
    number: u32,
}

#[derive(SerializeFields, Serialize, Deserialize, Debug, PartialEq)]
struct CollectionStruct {
    id: u32,
    items: Vec<InnerStruct>,
    tags: Vec<String>,
}

fn create_simple_struct() -> SimpleStruct {
    SimpleStruct {
        id: 123,
        name: "Test".to_string(),
        optional_field: Some("Optional".to_string()),
    }
}

fn create_nested_struct() -> NestedStruct {
    NestedStruct {
        id: 456,
        inner: InnerStruct {
            value: "Inner Value".to_string(),
            number: 42,
        },
        optional_inner: Some(InnerStruct {
            value: "Optional Inner".to_string(),
            number: 99,
        }),
    }
}

fn create_collection_struct() -> CollectionStruct {
    CollectionStruct {
        id: 789,
        items: vec![
            InnerStruct {
                value: "Item 1".to_string(),
                number: 1,
            },
            InnerStruct {
                value: "Item 2".to_string(),
                number: 2,
            },
        ],
        tags: vec!["tag1".to_string(), "tag2".to_string()],
    }
}

#[test]
fn test_utility_macros() {
    let data = create_nested_struct();
    let mut selector = data.serialize_fields();
    selector.enable_dot_hierarchy("id");
    selector.enable_dot_hierarchy("inner.value");

    // Test contains! macro
    assert!(contains!(selector, id));
    assert!(contains!(selector, inner.value));
    assert!(!contains!(selector, inner.number));
}

#[test]
fn test_macro_edge_cases() {
    let data = create_simple_struct();
    let selector = data.serialize_fields(); // No fields enabled

    // contains! should return false for empty selector
    assert!(!contains!(selector, id));
    assert!(!contains!(selector, name));

    // filter_field_set! should return empty set
    let empty_set = filter_field_set!(selector, {
        id => "id".to_string(),
        name => "name".to_string(),
    });
    assert!(empty_set.is_empty());

    // copy_selected_fields! should set all fields to None
    #[derive(Debug, PartialEq)]
    struct AllOptional {
        id: Option<u32>,
        name: Option<String>,
    }

    let all_none = copy_selected_fields!(selector, AllOptional {
        id: Some(data.id),
        name: Some(data.name.clone()),
    });

    assert_eq!(all_none.id, None);
    assert_eq!(all_none.name, None);
}

#[test]
fn test_serialize_fields_trait_api() {
    let data = create_simple_struct();
    
    // Test using the trait method
    let mut selector = data.serialize_fields();
    selector.enable_dot_hierarchy("id");
    selector.enable_dot_hierarchy("name");

    let json = serde_json::to_string(&SerializeFields(&data, &selector)).unwrap();
    let value: Value = serde_json::from_str(&json).unwrap();
    let obj = value.as_object().unwrap();

    assert_eq!(obj.len(), 2);
    assert_eq!(obj.get("id").unwrap().as_u64().unwrap(), 123);
    assert_eq!(obj.get("name").unwrap().as_str().unwrap(), "Test");
    assert!(!obj.contains_key("optional_field"));
}

#[test]
fn test_serialize_fields_trait_nested() {
    let data = create_nested_struct();
    
    // Test trait method with nested fields
    let mut selector = data.serialize_fields();
    selector.enable_dot_hierarchy("id");
    selector.enable_dot_hierarchy("inner.value");

    let json = serde_json::to_string(&SerializeFields(&data, &selector)).unwrap();
    let value: Value = serde_json::from_str(&json).unwrap();
    let obj = value.as_object().unwrap();

    assert_eq!(obj.len(), 2);
    assert_eq!(obj.get("id").unwrap().as_u64().unwrap(), 456);
    
    let inner = obj.get("inner").unwrap().as_object().unwrap();
    assert_eq!(inner.len(), 1);
    assert_eq!(inner.get("value").unwrap().as_str().unwrap(), "Inner Value");
}

#[test]
fn test_simple_field_selection() {
    let data = create_simple_struct();
    let mut selector = SimpleStructSerializeFieldSelector::new();
    selector.enable_dot_hierarchy("id");
    selector.enable_dot_hierarchy("name");

    let json = serde_json::to_string(&SerializeFields(&data, &selector)).unwrap();
    let value: Value = serde_json::from_str(&json).unwrap();
    let obj = value.as_object().unwrap();

    assert_eq!(obj.len(), 2);
    assert_eq!(obj.get("id").unwrap().as_u64().unwrap(), 123);
    assert_eq!(obj.get("name").unwrap().as_str().unwrap(), "Test");
    assert!(!obj.contains_key("optional_field"));
}

#[test]
fn test_no_fields_selected() {
    let data = create_simple_struct();
    let selector = SimpleStructSerializeFieldSelector::new();

    let json = serde_json::to_string(&SerializeFields(&data, &selector)).unwrap();
    let value: Value = serde_json::from_str(&json).unwrap();
    let obj = value.as_object().unwrap();

    assert_eq!(obj.len(), 0);
}

#[test]
fn test_all_fields_selected() {
    let data = create_simple_struct();
    let mut selector = SimpleStructSerializeFieldSelector::new();
    selector.enable_dot_hierarchy("id");
    selector.enable_dot_hierarchy("name");
    selector.enable_dot_hierarchy("optional_field");

    let json = serde_json::to_string(&SerializeFields(&data, &selector)).unwrap();
    let value: Value = serde_json::from_str(&json).unwrap();
    let obj = value.as_object().unwrap();

    assert_eq!(obj.len(), 3);
    assert_eq!(obj.get("id").unwrap().as_u64().unwrap(), 123);
    assert_eq!(obj.get("name").unwrap().as_str().unwrap(), "Test");
    assert_eq!(obj.get("optional_field").unwrap().as_str().unwrap(), "Optional");
}

#[test]
fn test_nested_field_selection() {
    let data = create_nested_struct();
    let mut selector = NestedStructSerializeFieldSelector::new();
    selector.enable_dot_hierarchy("id");
    selector.enable_dot_hierarchy("inner.value");

    let json = serde_json::to_string(&SerializeFields(&data, &selector)).unwrap();
    let value: Value = serde_json::from_str(&json).unwrap();
    let obj = value.as_object().unwrap();

    assert_eq!(obj.len(), 2);
    assert_eq!(obj.get("id").unwrap().as_u64().unwrap(), 456);
    
    let inner = obj.get("inner").unwrap().as_object().unwrap();
    assert_eq!(inner.len(), 1);
    assert_eq!(inner.get("value").unwrap().as_str().unwrap(), "Inner Value");
    assert!(!inner.contains_key("number"));
}

#[test]
fn test_nested_all_fields() {
    let data = create_nested_struct();
    let mut selector = NestedStructSerializeFieldSelector::new();
    selector.enable_dot_hierarchy("id");
    selector.enable_dot_hierarchy("inner.value");
    selector.enable_dot_hierarchy("inner.number");
    selector.enable_dot_hierarchy("optional_inner.value");

    let json = serde_json::to_string(&SerializeFields(&data, &selector)).unwrap();
    let value: Value = serde_json::from_str(&json).unwrap();
    let obj = value.as_object().unwrap();

    assert_eq!(obj.len(), 3);
    
    let inner = obj.get("inner").unwrap().as_object().unwrap();
    assert_eq!(inner.len(), 2);
    assert_eq!(inner.get("value").unwrap().as_str().unwrap(), "Inner Value");
    assert_eq!(inner.get("number").unwrap().as_u64().unwrap(), 42);
    
    let optional_inner = obj.get("optional_inner").unwrap().as_object().unwrap();
    assert_eq!(optional_inner.len(), 1);
    assert_eq!(optional_inner.get("value").unwrap().as_str().unwrap(), "Optional Inner");
}

#[test]
fn test_collection_field_selection() {
    let data = create_collection_struct();
    let mut selector = CollectionStructSerializeFieldSelector::new();
    selector.enable_dot_hierarchy("id");
    selector.enable_dot_hierarchy("items.value");
    selector.enable_dot_hierarchy("tags");

    let json = serde_json::to_string(&SerializeFields(&data, &selector)).unwrap();
    let value: Value = serde_json::from_str(&json).unwrap();
    let obj = value.as_object().unwrap();

    assert_eq!(obj.len(), 3);
    assert_eq!(obj.get("id").unwrap().as_u64().unwrap(), 789);
    
    let items = obj.get("items").unwrap().as_array().unwrap();
    assert_eq!(items.len(), 2);
    
    let item1 = items[0].as_object().unwrap();
    assert_eq!(item1.len(), 1);
    assert_eq!(item1.get("value").unwrap().as_str().unwrap(), "Item 1");
    assert!(!item1.contains_key("number"));
    
    let tags = obj.get("tags").unwrap().as_array().unwrap();
    assert_eq!(tags.len(), 2);
    assert_eq!(tags[0].as_str().unwrap(), "tag1");
    assert_eq!(tags[1].as_str().unwrap(), "tag2");
}

#[test]
fn test_enable_method_with_hierarchy() {
    let data = create_nested_struct();
    let mut selector = NestedStructSerializeFieldSelector::new();
    selector.enable(&["id"]);
    selector.enable(&["inner", "value"]);

    let json = serde_json::to_string(&SerializeFields(&data, &selector)).unwrap();
    let value: Value = serde_json::from_str(&json).unwrap();
    let obj = value.as_object().unwrap();

    assert_eq!(obj.len(), 2);
    assert_eq!(obj.get("id").unwrap().as_u64().unwrap(), 456);
    
    let inner = obj.get("inner").unwrap().as_object().unwrap();
    assert_eq!(inner.len(), 1);
    assert_eq!(inner.get("value").unwrap().as_str().unwrap(), "Inner Value");
}

#[test]
fn test_field_selector_trait() {
    // Test that the FieldSelector trait is implemented
    let mut selector = SimpleStructSerializeFieldSelector::new();
    
    // Test trait methods
    selector.enable_dot_hierarchy("id");
    selector.enable(&["name"]);
    
    // Verify fields are enabled
    assert!(selector.id.is_some());
    assert!(selector.name.is_some());
    assert!(selector.optional_field.is_none());
}

#[test]
fn test_utility_functions() {
    use serialize_fields::utils;
    
    // Test parse_field_list
    let fields = utils::parse_field_list("id,name,inner.value");
    assert_eq!(fields, vec!["id", "name", "inner.value"]);
    
    // Test create_selector_from_list
    let selector: SimpleStructSerializeFieldSelector = 
        utils::create_selector_from_list("id,name");
    
    assert!(selector.id.is_some());
    assert!(selector.name.is_some());
    assert!(selector.optional_field.is_none());
}

#[test]
fn test_empty_field_hierarchy() {
    let mut selector = SimpleStructSerializeFieldSelector::new();
    
    // Empty hierarchy should not panic
    selector.enable(&[]);
    
    // Verify no fields are enabled
    assert!(selector.id.is_none());
    assert!(selector.name.is_none());
    assert!(selector.optional_field.is_none());
}

#[test]
fn test_invalid_field_names() {
    let mut selector = SimpleStructSerializeFieldSelector::new();
    
    // Invalid field names should be silently ignored
    selector.enable_dot_hierarchy("nonexistent_field");
    selector.enable_dot_hierarchy("id.invalid_nested");
    
    // Valid field should still work
    selector.enable_dot_hierarchy("id");
    
    assert!(selector.id.is_some());
    assert!(selector.name.is_none());
}

#[test]
fn test_json_roundtrip_compatibility() {
    let original = create_simple_struct();
    
    // Serialize with all fields enabled
    let mut selector = SimpleStructSerializeFieldSelector::new();
    selector.enable_dot_hierarchy("id");
    selector.enable_dot_hierarchy("name");
    selector.enable_dot_hierarchy("optional_field");
    
    let json = serde_json::to_string(&SerializeFields(&original, &selector)).unwrap();
    
    // Should be able to deserialize back to original struct
    let deserialized: SimpleStruct = serde_json::from_str(&json).unwrap();
    assert_eq!(original, deserialized);
}

#[test]
fn test_field_enum_enable() {
    let data = create_simple_struct();
    let mut selector = SimpleStructSerializeFieldSelector::new();

    // Use enable_enum instead of string-based enable
    selector.enable_enum(SimpleStructField::Id);
    selector.enable_enum(SimpleStructField::Name);

    let json = serde_json::to_string(&SerializeFields(&data, &selector)).unwrap();
    let value: Value = serde_json::from_str(&json).unwrap();
    let obj = value.as_object().unwrap();

    assert_eq!(obj.len(), 2);
    assert_eq!(obj.get("id").unwrap().as_u64().unwrap(), 123);
    assert_eq!(obj.get("name").unwrap().as_str().unwrap(), "Test");
    assert!(!obj.contains_key("optional_field"));
}

#[test]
fn test_field_enum_nested_enable() {
    let data = create_nested_struct();
    let mut selector = NestedStructSerializeFieldSelector::new();

    // Use enable_enum with nested fields
    selector.enable_enum(NestedStructField::Id);
    selector.enable_enum(NestedStructField::Inner(InnerStructField::Value));
    selector.enable_enum(NestedStructField::OptionalInner(InnerStructField::Number));

    let json = serde_json::to_string(&SerializeFields(&data, &selector)).unwrap();
    let value: Value = serde_json::from_str(&json).unwrap();
    let obj = value.as_object().unwrap();

    assert_eq!(obj.len(), 3);
    assert_eq!(obj.get("id").unwrap().as_u64().unwrap(), 456);

    let inner = obj.get("inner").unwrap().as_object().unwrap();
    assert_eq!(inner.len(), 1);
    assert_eq!(inner.get("value").unwrap().as_str().unwrap(), "Inner Value");

    let optional_inner = obj.get("optional_inner").unwrap().as_object().unwrap();
    assert_eq!(optional_inner.len(), 1);
    assert_eq!(optional_inner.get("number").unwrap().as_u64().unwrap(), 99);
}

#[test]
fn test_field_enum_as_dot_path() {
    // Simple fields
    assert_eq!(SimpleStructField::Id.as_dot_path(), "id");
    assert_eq!(SimpleStructField::Name.as_dot_path(), "name");
    assert_eq!(SimpleStructField::OptionalField.as_dot_path(), "optional_field");

    // Nested fields
    assert_eq!(NestedStructField::Id.as_dot_path(), "id");
    assert_eq!(
        NestedStructField::Inner(InnerStructField::Value).as_dot_path(),
        "inner.value"
    );
    assert_eq!(
        NestedStructField::Inner(InnerStructField::Number).as_dot_path(),
        "inner.number"
    );
    assert_eq!(
        NestedStructField::OptionalInner(InnerStructField::Value).as_dot_path(),
        "optional_inner.value"
    );
}

#[test]
fn test_field_enum_display() {
    // Display should match as_dot_path
    assert_eq!(format!("{}", SimpleStructField::Id), "id");
    assert_eq!(
        format!("{}", NestedStructField::Inner(InnerStructField::Value)),
        "inner.value"
    );
}

#[test]
fn test_field_enum_from_str() {
    // Simple fields
    assert_eq!("id".parse::<SimpleStructField>().unwrap(), SimpleStructField::Id);
    assert_eq!("name".parse::<SimpleStructField>().unwrap(), SimpleStructField::Name);
    assert_eq!(
        "optional_field".parse::<SimpleStructField>().unwrap(),
        SimpleStructField::OptionalField
    );

    // Nested fields
    assert_eq!("id".parse::<NestedStructField>().unwrap(), NestedStructField::Id);
    assert_eq!(
        "inner.value".parse::<NestedStructField>().unwrap(),
        NestedStructField::Inner(InnerStructField::Value)
    );
    assert_eq!(
        "inner.number".parse::<NestedStructField>().unwrap(),
        NestedStructField::Inner(InnerStructField::Number)
    );
    assert_eq!(
        "optional_inner.value".parse::<NestedStructField>().unwrap(),
        NestedStructField::OptionalInner(InnerStructField::Value)
    );

    // Invalid field should return error
    assert!("nonexistent".parse::<SimpleStructField>().is_err());
    assert!("inner.nonexistent".parse::<NestedStructField>().is_err());
}

#[test]
fn test_field_enum_serde_roundtrip() {
    // Simple field
    let field = SimpleStructField::Name;
    let json = serde_json::to_string(&field).unwrap();
    assert_eq!(json, "\"name\"");
    let parsed: SimpleStructField = serde_json::from_str(&json).unwrap();
    assert_eq!(parsed, field);

    // Nested field
    let nested_field = NestedStructField::Inner(InnerStructField::Value);
    let json = serde_json::to_string(&nested_field).unwrap();
    assert_eq!(json, "\"inner.value\"");
    let parsed: NestedStructField = serde_json::from_str(&json).unwrap();
    assert_eq!(parsed, nested_field);
}

#[test]
fn test_field_enum_serde_deserialize_from_string() {
    // Deserialize field enums from JSON strings (mimics string behavior)
    let field: SimpleStructField = serde_json::from_str("\"id\"").unwrap();
    assert_eq!(field, SimpleStructField::Id);

    let field: NestedStructField = serde_json::from_str("\"inner.number\"").unwrap();
    assert_eq!(field, NestedStructField::Inner(InnerStructField::Number));

    // Array of field enums
    let fields: Vec<SimpleStructField> = serde_json::from_str("[\"id\", \"name\"]").unwrap();
    assert_eq!(fields, vec![SimpleStructField::Id, SimpleStructField::Name]);

    let fields: Vec<NestedStructField> =
        serde_json::from_str("[\"id\", \"inner.value\", \"optional_inner.number\"]").unwrap();
    assert_eq!(fields, vec![
        NestedStructField::Id,
        NestedStructField::Inner(InnerStructField::Value),
        NestedStructField::OptionalInner(InnerStructField::Number),
    ]);
}

#[cfg(feature = "schemars")]
#[test]
fn test_field_enum_json_schema() {
    // Verify that JsonSchema is implemented for field enums
    let schema = schemars::schema_for!(SimpleStructField);
    let json = serde_json::to_string_pretty(&schema).unwrap();

    // Schema should describe a string type with enum values
    assert!(json.contains("\"type\": \"string\""));
    assert!(json.contains("SimpleStructField"));
    assert!(json.contains("\"enum\""));
    assert!(json.contains("\"id\""));
    assert!(json.contains("\"name\""));
    assert!(json.contains("\"optional_field\""));

    // Nested enum should also have schema with prefixed nested values
    let nested_schema = schemars::schema_for!(NestedStructField);
    let nested_json = serde_json::to_string_pretty(&nested_schema).unwrap();
    assert!(nested_json.contains("\"type\": \"string\""));
    assert!(nested_json.contains("\"enum\""));
    assert!(nested_json.contains("\"id\""));
    // Nested fields should be prefixed with their parent field name
    assert!(nested_json.contains("\"inner.value\""));
    assert!(nested_json.contains("\"inner.number\""));
    assert!(nested_json.contains("\"optional_inner.value\""));
    assert!(nested_json.contains("\"optional_inner.number\""));
}