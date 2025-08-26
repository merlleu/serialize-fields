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

    for field in fields {
        let field_ident = field.ident.as_ref().unwrap();
        
        // Handle raw identifiers (r#keyword)
        let field_name_str = strip_raw_prefix(&field_ident.to_string());
        
        // Create a safe field name for the selector struct (can't use keywords)
        let field_ident = field_ident;

        // Determine if this is a nested struct type that would have SerializeFields
        let (is_nested, nested_type) = analyze_field_type(&field.ty);

        if is_nested {
            let nested_selector_type = syn::Ident::new(
                &format!("{}SerializeFieldSelector", nested_type),
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
        }

        new_field_inits.push(quote! {
            #field_ident: None
        });
    }

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

    // Generate the complete implementation
    let expanded = quote! {
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