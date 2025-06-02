//! Procedural macros for the `serialize_fields` crate.
//!
//! This crate provides the `SerializeFields` derive macro that generates
//! field selectors and serialization logic for dynamic field selection.

use proc_macro::TokenStream;
use quote::quote;
use syn::{
    parse_macro_input, Data, DeriveInput, Fields, GenericArgument, PathArguments, Type,
};

/// Derive macro for generating field selectors and dynamic serialization.
///
/// This macro generates:
/// - A field selector struct (`{StructName}SerializeFieldSelector`)
/// - Methods for enabling fields by hierarchy
/// - `Serialize` implementations that respect field selection
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
/// }
/// ```
///
/// This generates:
/// - `UserSerializeFieldSelector` struct
/// - Methods: `new()`, `enable_dot_hierarchy()`, `enable()`
/// - `Serialize` impl for `SerializeFields<User, UserSerializeFieldSelector>`
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
        let field_name = field.ident.as_ref().unwrap();
        let field_name_str = field_name.to_string();
        
        // Determine if this is a nested struct type that would have SerializeFields
        let (is_nested, nested_type) = analyze_field_type(&field.ty);
        
        if is_nested {
            let nested_selector_type = syn::Ident::new(
                &format!("{}SerializeFieldSelector", nested_type),
                field_name.span(),
            );
            
            selector_fields.push(quote! {
                pub #field_name: Option<#nested_selector_type>
            });
            
            enable_match_arms.push(quote! {
                #field_name_str => {
                    match &mut self.#field_name {
                        Some(ref mut nested) => nested.enable(&field_hierarchy[1..]),
                        None => {
                            let mut new_nested = #nested_selector_type::new();
                            new_nested.enable(&field_hierarchy[1..]);
                            self.#field_name = Some(new_nested);
                        }
                    }
                }
            });
            
            serialize_fields.push(quote! {
                if let Some(ref nested_selector) = field_selector.#field_name {
                    state.serialize_field(#field_name_str, &SerializeFields(&data.#field_name, nested_selector))?;
                }
            });
        } else {
            selector_fields.push(quote! {
                pub #field_name: Option<()>
            });
            
            enable_match_arms.push(quote! {
                #field_name_str => self.#field_name = Some(())
            });
            
            serialize_fields.push(quote! {
                if field_selector.#field_name.is_some() {
                    state.serialize_field(#field_name_str, &data.#field_name)?;
                }
            });
        }
        
        new_field_inits.push(quote! {
            #field_name: None
        });
    }
    
    // Count enabled fields for serialization
    let count_enabled_fields = fields.iter().map(|field| {
        let field_name = field.ident.as_ref().unwrap();
        quote! {
            + if field_selector.#field_name.is_some() { 1 } else { 0 }
        }
    }).collect::<Vec<_>>();
    
    // Generate the complete implementation
    let expanded = quote! {
        #[derive(Debug, Clone)]
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
        }
        
        // SerializeFields implementation for this specific struct
        impl<'a> ::serde::Serialize for ::serialize_fields::SerializeFields<'a, #struct_name, #selector_ident> {
            fn serialize<Se>(&self, serializer: Se) -> Result<Se::Ok, Se::Error>
            where
                Se: ::serde::Serializer,
            {
                use ::serde::ser::SerializeStruct;
                use ::serialize_fields::SerializeFields;
                
                let data = self.0;
                let field_selector = self.1;
                
                // Count enabled fields
                let field_count = 0 #(#count_enabled_fields)*;
                
                let mut state = serializer.serialize_struct(stringify!(#struct_name), field_count)?;
                
                #(#serialize_fields)*
                
                state.end()
            }
        }
    };
    
    // Also generate Vec serialization support
    let vec_impl = generate_vec_serialization(&struct_name, &selector_ident);
    let option_impl = generate_option_serialization(&struct_name, &selector_ident);
    
    let final_output = quote! {
        #expanded
        #vec_impl
        #option_impl
    };
    
    TokenStream::from(final_output)
}

/// Generate serialization support for Vec<T> where T is our struct
fn generate_vec_serialization(
    struct_name: &syn::Ident, 
    selector_ident: &syn::Ident
) -> proc_macro2::TokenStream {
    quote! {
        impl<'a> ::serde::Serialize for ::serialize_fields::SerializeFields<'a, Vec<#struct_name>, #selector_ident> {
            fn serialize<Se>(&self, serializer: Se) -> Result<Se::Ok, Se::Error>
            where
                Se: ::serde::Serializer,
            {
                use ::serde::ser::SerializeSeq;
                use ::serialize_fields::SerializeFields;
                
                let data = self.0;
                let field_selector = self.1;
                
                let mut seq = serializer.serialize_seq(Some(data.len()))?;
                
                for item in data {
                    seq.serialize_element(&SerializeFields(item, field_selector))?;
                }
                
                seq.end()
            }
        }
    }
}

/// Generate serialization support for Option<T> where T is our struct
fn generate_option_serialization(
    struct_name: &syn::Ident, 
    selector_ident: &syn::Ident
) -> proc_macro2::TokenStream {
    quote! {
        impl<'a> ::serde::Serialize for ::serialize_fields::SerializeFields<'a, Option<#struct_name>, #selector_ident> {
            fn serialize<Se>(&self, serializer: Se) -> Result<Se::Ok, Se::Error>
            where
                Se: ::serde::Serializer,
            {
                use ::serialize_fields::SerializeFields;
                
                let data = self.0;
                let field_selector = self.1;
                
                match data {
                    Some(ref inner) => SerializeFields(inner, field_selector).serialize(serializer),
                    None => serializer.serialize_none(),
                }
            }
        }
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
                "u8" | "u16" | "u32" | "u64" | "u128" | "usize" |
                "i8" | "i16" | "i32" | "i64" | "i128" | "isize" |
                "f32" | "f64" | "bool" | "char" | "String" => (false, String::new()),
                
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
        Type::Tuple(type_tuple) => {
            // For tuples, assume they're not custom structs
            (false, String::new())
        }
        _ => (false, String::new()),
    }
}