//! Procedural macros for the `serialize_fields` crate.
//!
//! This crate provides the `SerializeFields` derive macro that generates
//! field selectors and serialization logic for dynamic field selection.

use proc_macro::TokenStream;
use quote::quote;
use syn::{Data, DeriveInput, Fields, GenericArgument, PathArguments, Type, parse_macro_input};

/// Derive macro for generating field selectors and dynamic serialization.
///
/// This macro generates:
/// - A field selector struct (`{StructName}SerializeFieldSelector`)
/// - Methods for enabling fields by hierarchy  
/// - Implementation of `SerializeFieldsTrait` with serialization logic
///
/// The serialization is handled by a generic `Serialize` implementation that
/// works with any type implementing `SerializeFieldsTrait`.
///
/// # Examples
///
/// ```rust
/// use serialize_fields::SerializeFields;
/// use serde::{Serialize, Deserialize};
///
/// #[derive(SerializeFields, Serialize, Deserialize)]
/// struct User {
///     id: u32,
///     name: String,
///     email: Option<String>,
///     r#type: String, // Raw identifier support
/// }
/// ```
///
/// This generates:
/// - `UserSerializeFieldSelector` struct
/// - Methods: `new()`, `enable_dot_hierarchy()`, `enable()`
/// - `SerializeFieldsTrait` impl with `serialize_fields()` and `serialize()` methods
#[proc_macro_derive(SerializeFields)]
pub fn serialize_fields_derive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let struct_name = &input.ident;
    let selector_name = format!("{}SerializeFieldSelector", struct_name);
    let selector_ident = syn::Ident::new(&selector_name, struct_name.span());

    // Parse fields
    let fields = match &input.data {
        Data::Struct(data) => match &data.fields {
            Fields::Named(fields) => &fields.named,
            _ => panic!("SerializeFields only supports structs with named fields"),
        },
        _ => panic!("SerializeFields only supports structs"),
    };

    // Generate field selector struct fields
    let mut selector_fields = Vec::new();
    let mut enable_match_arms = Vec::new();
    let mut new_field_inits = Vec::new();
    let mut serialize_fields = Vec::new();

    // Generate field enum
    let field_enum_name = format!("{}Field", struct_name);
    let field_enum_ident = syn::Ident::new(&field_enum_name, struct_name.span());
    let mut enum_variants = Vec::new();
    let mut enable_enum_match_arms = Vec::new();
    let mut as_dot_path_arms = Vec::new();
    let mut deserialize_match_arms = Vec::new();
    #[cfg(feature = "schemars")]
    let mut schema_simple_fields: Vec<String> = Vec::new();
    #[cfg(feature = "schemars")]
    let mut schema_nested_prefixes: Vec<(String, String)> = Vec::new(); // (prefix, nested_type)

    for field in fields {
        let field_ident = field.ident.as_ref().unwrap();
        
        // Handle raw identifiers (r#keyword)
        let field_name_str = strip_raw_prefix(&field_ident.to_string());
        
        // Create a safe field name for the selector struct (can't use keywords)
        let field_ident = field_ident;

        // Determine if this is a nested struct type that would have SerializeFields
        let (is_nested, nested_type) = analyze_field_type(&field.ty);

        // Create variant name (PascalCase from snake_case)
        let variant_name = to_pascal_case(&field_name_str);
        let variant_ident = syn::Ident::new(&variant_name, field_ident.span());

        if is_nested {
            let nested_selector_type = syn::Ident::new(
                &format!("{}SerializeFieldSelector", nested_type),
                field_ident.span(),
            );
            let nested_field_enum = syn::Ident::new(
                &format!("{}Field", nested_type),
                field_ident.span(),
            );

            selector_fields.push(quote! {
                #[serde(skip_serializing_if = "Option::is_none")]
                pub #field_ident: Option<#nested_selector_type>
            });

            enable_match_arms.push(quote! {
                #field_name_str => {
                    match &mut self.#field_ident {
                        Some(nested) => nested.enable(&field_hierarchy[1..]),
                        None => {
                            let mut new_nested = #nested_selector_type::new();
                            new_nested.enable(&field_hierarchy[1..]);
                            self.#field_ident = Some(new_nested);
                        }
                    }
                }
            });

            serialize_fields.push(quote! {
                if let Some(ref nested_selector) = field_selector.#field_ident {
                    state.serialize_field(#field_name_str, &SerializeFields(&data.#field_ident, nested_selector))?;
                }
            });

            // Enum variant with nested field
            enum_variants.push(quote! {
                #variant_ident(#nested_field_enum)
            });

            enable_enum_match_arms.push(quote! {
                #field_enum_ident::#variant_ident(nested) => {
                    match &mut self.#field_ident {
                        Some(selector) => {
                            selector.enable_enum(nested);
                        }
                        None => {
                            let mut new_nested = #nested_selector_type::new();
                            new_nested.enable_enum(nested);
                            self.#field_ident = Some(new_nested);
                        }
                    }
                }
            });

            as_dot_path_arms.push(quote! {
                #field_enum_ident::#variant_ident(nested) => {
                    format!("{}.{}", #field_name_str, nested.as_dot_path())
                }
            });

            deserialize_match_arms.push(quote! {
                s if s.starts_with(concat!(#field_name_str, ".")) => {
                    let rest = &s[#field_name_str.len() + 1..];
                    Ok(#field_enum_ident::#variant_ident(rest.parse()?))
                }
            });

            #[cfg(feature = "schemars")]
            schema_nested_prefixes.push((field_name_str.clone(), nested_type.clone()));
        } else {
            selector_fields.push(quote! {
                #[serde(skip_serializing_if = "Option::is_none")]
                pub #field_ident: Option<()>
            });

            enable_match_arms.push(quote! {
                #field_name_str => self.#field_ident = Some(())
            });

            serialize_fields.push(quote! {
                if field_selector.#field_ident.is_some() {
                    state.serialize_field(#field_name_str, &data.#field_ident)?;
                }
            });

            // Simple enum variant
            enum_variants.push(quote! {
                #variant_ident
            });

            enable_enum_match_arms.push(quote! {
                #field_enum_ident::#variant_ident => self.#field_ident = Some(())
            });

            as_dot_path_arms.push(quote! {
                #field_enum_ident::#variant_ident => #field_name_str.to_string()
            });

            deserialize_match_arms.push(quote! {
                #field_name_str => Ok(#field_enum_ident::#variant_ident)
            });

            #[cfg(feature = "schemars")]
            schema_simple_fields.push(field_name_str.clone());
        }

        new_field_inits.push(quote! {
            #field_ident: None
        });
    }

    // Generate schema nested field tokens (used only with schemars feature)
    #[cfg(feature = "schemars")]
    let schema_nested_enum_types: Vec<_> = schema_nested_prefixes
        .iter()
        .map(|(_, nested_type)| {
            let ident = syn::Ident::new(&format!("{}Field", nested_type), struct_name.span());
            quote! { #ident }
        })
        .collect();
    #[cfg(feature = "schemars")]
    let schema_nested_prefix_strs: Vec<_> = schema_nested_prefixes
        .iter()
        .map(|(prefix, _)| prefix.clone())
        .collect();

    // Count enabled fields for serialization
    let count_enabled_fields = fields
        .iter()
        .map(|field: &syn::Field| {
            let field_ident = field.ident.as_ref().unwrap();
            quote! {
                + if field_selector.#field_ident.is_some() { 1 } else { 0 }
            }
        })
        .collect::<Vec<_>>();

    // Generate schemars impl conditionally at macro compile-time
    #[cfg(feature = "schemars")]
    let schemars_impl = quote! {
        impl ::schemars::JsonSchema for #field_enum_ident {
            fn schema_name() -> ::std::borrow::Cow<'static, str> {
                ::std::borrow::Cow::Borrowed(stringify!(#field_enum_ident))
            }

            fn json_schema(generator: &mut ::schemars::SchemaGenerator) -> ::schemars::Schema {
                // Collect all possible enum values
                let mut all_values: Vec<String> = Vec::new();

                // Add simple field values
                #(all_values.push(#schema_simple_fields.to_string());)*

                // For nested fields, get their enum values and prefix them
                #(
                    // Call json_schema directly to get the inline schema, not a $ref
                    let nested_schema = <#schema_nested_enum_types as ::schemars::JsonSchema>::json_schema(generator);
                    if let Some(obj) = nested_schema.as_object() {
                        if let Some(enum_values) = obj.get("enum").and_then(|v| v.as_array()) {
                            for val in enum_values {
                                if let Some(s) = val.as_str() {
                                    all_values.push(format!("{}.{}", #schema_nested_prefix_strs, s));
                                }
                            }
                        }
                    }
                )*

                ::schemars::json_schema!({
                    "type": "string",
                    "enum": all_values,
                    "description": concat!("Field selector for ", stringify!(#struct_name), " - serializes as dot notation (e.g., \"field.nested\")")
                })
            }
        }
    };

    #[cfg(not(feature = "schemars"))]
    let schemars_impl = quote! {};

    // Generate the complete implementation
    let expanded = quote! {
        /// Enum representing all fields of `#struct_name` for type-safe field selection.
        /// Serializes to dot notation (e.g., "profile.bio").
        #[derive(Debug, Clone, PartialEq, Eq, Hash)]
        pub enum #field_enum_ident {
            #(#enum_variants,)*
        }

        impl #field_enum_ident {
            /// Returns the dot notation path for this field.
            pub fn as_dot_path(&self) -> String {
                match self {
                    #(#as_dot_path_arms,)*
                }
            }
        }

        impl ::std::fmt::Display for #field_enum_ident {
            fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
                write!(f, "{}", self.as_dot_path())
            }
        }

        impl ::std::str::FromStr for #field_enum_ident {
            type Err = String;

            fn from_str(s: &str) -> Result<Self, Self::Err> {
                match s {
                    #(#deserialize_match_arms,)*
                    _ => Err(format!("Unknown field: {}", s)),
                }
            }
        }

        impl ::serde::Serialize for #field_enum_ident {
            fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
            where
                S: ::serde::Serializer,
            {
                serializer.serialize_str(&self.as_dot_path())
            }
        }

        impl<'de> ::serde::Deserialize<'de> for #field_enum_ident {
            fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
            where
                D: ::serde::Deserializer<'de>,
            {
                let s = String::deserialize(deserializer)?;
                s.parse().map_err(::serde::de::Error::custom)
            }
        }

        #schemars_impl

        #[derive(Debug, Clone, PartialEq, Eq, Hash, ::serde::Serialize)]
        pub struct #selector_ident {
            #(#selector_fields,)*
        }

        impl #selector_ident {
            pub fn new() -> Self {
                #selector_ident {
                    #(#new_field_inits,)*
                }
            }

            pub fn enable_dot_hierarchy(&mut self, field: &str) {
                let split: Vec<&str> = field.split('.').collect();
                self.enable(&split);
            }

            pub fn enable(&mut self, field_hierarchy: &[&str]) {
                if field_hierarchy.is_empty() {
                    return;
                }

                match field_hierarchy[0] {
                    #(#enable_match_arms,)*
                    _ => {}
                }
            }

            /// Enable a field using the type-safe field enum.
            pub fn enable_enum(&mut self, field: #field_enum_ident) {
                match field {
                    #(#enable_enum_match_arms,)*
                }
            }
        }

        impl Default for #selector_ident {
            fn default() -> Self {
                Self::new()
            }
        }

        impl ::serialize_fields::FieldSelector for #selector_ident {
            fn new() -> Self {
                Self::new()
            }

            fn enable_dot_hierarchy(&mut self, field: &str) {
                self.enable_dot_hierarchy(field)
            }

            fn enable(&mut self, field_hierarchy: &[&str]) {
                self.enable(field_hierarchy)
            }
        }

        impl ::serialize_fields::SerializeFieldsTrait for #struct_name {
            type FieldSelector = #selector_ident;

            fn serialize_fields(&self) -> Self::FieldSelector {
                #selector_ident::new()
            }

            fn serialize<__S>(
                &self,
                field_selector: &Self::FieldSelector,
                __serializer: __S,
            ) -> Result<__S::Ok, __S::Error>
            where
                __S: ::serde::Serializer,
            {
                use ::serde::ser::SerializeStruct;
                use ::serialize_fields::SerializeFields;

                let data = self;

                // Count enabled fields
                let field_count = 0 #(#count_enabled_fields)*;

                let mut state = __serializer.serialize_struct(stringify!(#struct_name), field_count)?;

                #(#serialize_fields)*

                state.end()
            }
        }
    };

    TokenStream::from(expanded)
}

/// Strip the r# prefix from raw identifiers
fn strip_raw_prefix(s: &str) -> String {
    if s.starts_with("r#") {
        s[2..].to_string()
    } else {
        s.to_string()
    }
}

/// Convert snake_case to PascalCase for enum variant names
fn to_pascal_case(s: &str) -> String {
    s.split('_')
        .map(|word| {
            let mut chars = word.chars();
            match chars.next() {
                Some(first) => first.to_uppercase().chain(chars).collect(),
                None => String::new(),
            }
        })
        .collect()
}

/// Analyze a field type to determine if it's a nested struct and what type it is
fn analyze_field_type(ty: &Type) -> (bool, String) {
    match ty {
        Type::Path(type_path) => {
            let last_segment = type_path.path.segments.last().unwrap();
            let type_name = last_segment.ident.to_string();

            match type_name.as_str() {
                // Primitive types
                "u8" | "u16" | "u32" | "u64" | "u128" | "usize" | "i8" | "i16" | "i32" | "i64"
                | "i128" | "isize" | "f32" | "f64" | "bool" | "char" | "String" => {
                    (false, String::new())
                }

                // Standard library types that don't derive SerializeFields
                "PathBuf" | "SystemTime" | "Duration" => (false, String::new()),

                // Container types - check inner type for Vec, Option, etc.
                "Option" | "Vec" | "HashMap" | "BTreeMap" | "HashSet" | "BTreeSet" => {
                    if let PathArguments::AngleBracketed(args) = &last_segment.arguments {
                        if let Some(GenericArgument::Type(inner_ty)) = args.args.first() {
                            return analyze_field_type(inner_ty);
                        }
                    }
                    (false, String::new())
                }

                // Result and similar types - usually not serialized
                "Result" | "Box" | "Rc" | "Arc" => (false, String::new()),

                // Assume any other type is a custom struct that might derive SerializeFields
                _ => (true, type_name),
            }
        }
        Type::Array(type_array) => {
            // For arrays like [T; N], check the element type
            analyze_field_type(&type_array.elem)
        }
        Type::Tuple(_type_tuple) => {
            // For tuples, assume they're not custom structs
            (false, String::new())
        }
        _ => (false, String::new()),
    }
}