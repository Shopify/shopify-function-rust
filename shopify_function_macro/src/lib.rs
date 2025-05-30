use std::collections::HashMap;

use bluejay_core::{
    definition::{
        EnumTypeDefinition, EnumValueDefinition, InputObjectTypeDefinition, InputValueDefinition,
    },
    AsIter,
};
use bluejay_typegen_codegen::{
    generate_schema, names, CodeGenerator, ExecutableStruct, Input as BluejayInput,
    KnownCustomScalarType, WrappedExecutableType,
};
use convert_case::{Case, Casing};
use proc_macro2::Span;
use quote::{format_ident, quote, ToTokens};
use syn::{parse_macro_input, parse_quote, FnArg};

fn extract_shopify_function_return_type(ast: &syn::ItemFn) -> Result<&syn::Ident, syn::Error> {
    use syn::*;

    let ReturnType::Type(_arrow, ty) = &ast.sig.output else {
        return Err(Error::new_spanned(
            &ast.sig,
            "Shopify Functions require an explicit return type",
        ));
    };
    let Type::Path(path) = ty.as_ref() else {
        return Err(Error::new_spanned(
            &ast.sig,
            "Shopify Functions must return a Result",
        ));
    };
    let result = path.path.segments.last().unwrap();
    if result.ident != "Result" {
        return Err(Error::new_spanned(
            result,
            "Shopify Functions must return a Result",
        ));
    }
    let PathArguments::AngleBracketed(generics) = &result.arguments else {
        return Err(Error::new_spanned(
            result,
            "Shopify Function Result is missing generic arguments",
        ));
    };
    if generics.args.len() != 1 {
        return Err(Error::new_spanned(
            generics,
            "Shopify Function Result takes exactly one generic argument",
        ));
    }
    let GenericArgument::Type(ty) = generics.args.first().unwrap() else {
        return Err(Error::new_spanned(
            generics,
            "Shopify Function Result expects a type",
        ));
    };
    let Type::Path(path) = ty else {
        return Err(Error::new_spanned(
            result,
            "Unexpected result type for Shopify Function Result",
        ));
    };
    Ok(&path.path.segments.last().as_ref().unwrap().ident)
}

/// Generates code for a Function. This will define a wrapper function that is exported to Wasm.
/// The wrapper handles deserializing the input and serializing the output.
#[proc_macro_attribute]
pub fn shopify_function(
    attr: proc_macro::TokenStream,
    item: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    let ast = parse_macro_input!(item as syn::ItemFn);
    if !attr.is_empty() {
        return quote! {compile_error!("Shopify functions don't accept attributes");}.into();
    }

    let function_name = &ast.sig.ident;
    let function_name_string = function_name.to_string();
    let export_function_name = format_ident!("{}_export", function_name);

    if ast.sig.inputs.len() != 1 {
        return quote! {compile_error!("Shopify functions need exactly one input parameter");}
            .into();
    }

    let input_type = match &ast.sig.inputs.first().unwrap() {
        FnArg::Typed(input) => input.ty.as_ref(),
        FnArg::Receiver(_) => {
            return quote! {compile_error!("Shopify functions can't have a receiver");}.into()
        }
    };

    if let Err(error) = extract_shopify_function_return_type(&ast) {
        return error.to_compile_error().into();
    }

    quote! {
        #[export_name = #function_name_string]
        pub extern "C" fn #export_function_name() {
            let mut context = shopify_function::wasm_api::Context::new();
            let root_value = context.input_get().expect("Failed to get input");
            let mut input: #input_type = shopify_function::wasm_api::Deserialize::deserialize(&root_value).expect("Failed to deserialize input");
            let result = #function_name(input).expect("Failed to call function");
            shopify_function::wasm_api::Serialize::serialize(&result, &mut context).expect("Failed to serialize output");
            context.finalize_output().expect("Failed to finalize output");
        }

        #ast
    }
    .into()
}

const DEFAULT_EXTERN_ENUMS: &[&str] = &["LanguageCode", "CountryCode", "CurrencyCode"];

mod kw {
    syn::custom_keyword!(input_stream);
    syn::custom_keyword!(output_stream);
}

/// Generates Rust types from GraphQL schema definitions and queries.
///
/// ### Arguments
///
/// **Positional:**
///
/// 1. String literal with path to the file containing the schema definition. If relative, should be with respect to
///    the project root (wherever `Cargo.toml` is located).
///
/// **Optional keyword:**
///
/// _enums_as_str_: Optional list of enum names for which the generated code should use string types instead of
/// a fully formed enum. Defaults to `["LanguageCode", "CountryCode", "CurrencyCode"]`.
///
/// ### Trait implementations
///
/// By default, will implement `PartialEq`, and `Debug` for all input and enum types. Enums will also implement `Copy`.
/// For types corresponding to values returned from queries,  the `shopify_function::wasm_api::Deserialize` trait
/// is implemented. For types that would
/// be arguments to a query, the `shopify_function::wasm_api::Serialize` trait is implemented.
///
/// ### Usage
///
/// Must be used with a module. Inside the module, type aliases must be defined for any custom scalars in the schema.
///
/// #### Queries
///
/// To use a query, define a module within the aforementioned module, and annotate it with
/// `#[query("path/to/query.graphql")]`, where the argument is a string literal path to the query document, or the
/// query contents enclosed in square brackets.
///
/// ##### Custom scalar overrides
///
/// To override the type of a custom scalar for a path within a query, use the `custom_scalar_overrides` named argument
/// inside of the `#[query(...)]` attribute. The argument is a map from a path to a type, where the path is a string literal
/// path to the field in the query, and the type is the type to override the field with.
///
/// For example, with the following query:
/// ```graphql
/// query MyQuery {
///     myField: myScalar!
/// }
/// ```
/// do something like the following:
/// ```ignore
/// #[query("path/to/query.graphql", custom_scalar_overrides = {
///     "MyQuery.myField" => ::std::primitive::i32,
/// })]
/// ```
/// Any type path that does not start with `::` is assumed to be relative to the schema definition module.
///
/// ### Naming
///
/// To generate idiomatic Rust code, some renaming of types, enum variants, and fields is performed. Types are
/// renamed with `PascalCase`, as are enum variants. Fields are renamed with `snake_case`.
///
/// ### Query restrictions
///
/// In order to keep the type generation code relatively simple, there are some restrictions on the queries that are
/// permitted. This may be relaxed in future versions.
/// * Selection sets on object and interface types must contain either a single fragment spread, or entirely field
///   selections.
/// * Selection sets on union types must contain either a single fragment spread, or both an unaliased `__typename`
///   selection and inline fragments for all or a subset of the objects contained in the union.
#[proc_macro_attribute]
pub fn typegen(
    attr: proc_macro::TokenStream,
    item: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    let mut input = syn::parse_macro_input!(attr as BluejayInput);
    let mut module = syn::parse_macro_input!(item as syn::ItemMod);

    if let Some(borrow) = input.borrow.as_ref() {
        if borrow.value() {
            let error = syn::Error::new_spanned(
                borrow,
                "`borrow` attribute must be `false` or omitted for Shopify Functions",
            );
            return error.to_compile_error().into();
        }
    }

    if input.enums_as_str.is_empty() {
        let enums_as_str = DEFAULT_EXTERN_ENUMS
            .iter()
            .map(|enum_name| syn::LitStr::new(enum_name, Span::mixed_site()))
            .collect::<Vec<_>>();
        input.enums_as_str = syn::parse_quote! { #(#enums_as_str),* };
    }

    let string_known_custom_scalar_type = KnownCustomScalarType {
        type_for_borrowed: None, // we disallow borrowing
        type_for_owned: syn::parse_quote! { ::std::string::String },
    };

    let known_custom_scalar_types = HashMap::from([
        (String::from("Id"), string_known_custom_scalar_type.clone()),
        (String::from("Url"), string_known_custom_scalar_type.clone()),
        (
            String::from("Handle"),
            string_known_custom_scalar_type.clone(),
        ),
        (
            String::from("Date"),
            string_known_custom_scalar_type.clone(),
        ),
        (
            String::from("DateTime"),
            string_known_custom_scalar_type.clone(),
        ),
        (
            String::from("DateTimeWithoutTimezone"),
            string_known_custom_scalar_type.clone(),
        ),
        (
            String::from("TimeWithoutTimezone"),
            string_known_custom_scalar_type.clone(),
        ),
        (
            String::from("Void"),
            KnownCustomScalarType {
                type_for_borrowed: None,
                type_for_owned: syn::parse_quote! { () },
            },
        ),
        (
            String::from("Json"),
            KnownCustomScalarType {
                type_for_borrowed: None,
                type_for_owned: syn::parse_quote! { ::shopify_function::scalars::JsonValue },
            },
        ),
        (
            String::from("Decimal"),
            KnownCustomScalarType {
                type_for_borrowed: None,
                type_for_owned: syn::parse_quote! { ::shopify_function::scalars::Decimal },
            },
        ),
    ]);

    if let Err(error) = generate_schema(
        input,
        &mut module,
        known_custom_scalar_types,
        ShopifyFunctionCodeGenerator,
    ) {
        return error.to_compile_error().into();
    }

    module.to_token_stream().into()
}

struct ShopifyFunctionCodeGenerator;

impl CodeGenerator for ShopifyFunctionCodeGenerator {
    fn fields_for_executable_struct(
        &self,
        executable_struct: &bluejay_typegen_codegen::ExecutableStruct,
    ) -> syn::Fields {
        let once_cell_fields: Vec<syn::Field> = executable_struct
            .fields()
            .iter()
            .map(|field| {
                let field_name_ident = names::field_ident(field.graphql_name());
                let field_type = Self::type_for_field(executable_struct, field.r#type(), false);

                parse_quote! {
                    #field_name_ident: ::std::cell::OnceCell<#field_type>
                }
            })
            .collect();

        let fields_named: syn::FieldsNamed = parse_quote! {
            {
                __wasm_value: shopify_function::wasm_api::Value,
                #(#once_cell_fields),*
            }
        };
        fields_named.into()
    }

    fn additional_impls_for_executable_struct(
        &self,
        executable_struct: &bluejay_typegen_codegen::ExecutableStruct,
    ) -> Vec<syn::ItemImpl> {
        let name_ident = names::type_ident(executable_struct.parent_name());

        let once_cell_field_values: Vec<syn::FieldValue> = executable_struct
            .fields()
            .iter()
            .map(|field| {
                let field_name_ident = names::field_ident(field.graphql_name());

                parse_quote! {
                    #field_name_ident: ::std::cell::OnceCell::new()
                }
            })
            .collect();

        let deserialize_impl = parse_quote! {
            impl shopify_function::wasm_api::Deserialize for #name_ident {
                fn deserialize(value: &shopify_function::wasm_api::Value) -> ::std::result::Result<Self, shopify_function::wasm_api::read::Error> {
                    Ok(Self {
                        __wasm_value: *value,
                        #(#once_cell_field_values),*
                    })
                }
            }
        };

        let accessors: Vec<syn::ImplItemFn> = executable_struct
            .fields()
            .iter()
            .map(|field| {
                let field_name_ident = names::field_ident(field.graphql_name());
                let field_name_lit_str = syn::LitStr::new(field.graphql_name(), Span::mixed_site());
                let field_type = Self::type_for_field(executable_struct, field.r#type(), true);

                let properly_referenced_value =
                    Self::reference_variable_for_type(field.r#type(), &format_ident!("value_ref"));

                let description: Option<syn::Attribute> = field.description().map(|description| {
                    let description_lit_str = syn::LitStr::new(description, Span::mixed_site());
                    parse_quote! { #[doc = #description_lit_str] }
                });

                parse_quote! {
                    #description
                    pub fn #field_name_ident(&self) -> #field_type {
                        static INTERNED_FIELD_NAME: shopify_function::wasm_api::CachedInternedStringId = shopify_function::wasm_api::CachedInternedStringId::new(#field_name_lit_str, );
                        let interned_string_id = INTERNED_FIELD_NAME.load_from_value(&self.__wasm_value);

                        let value = self.#field_name_ident.get_or_init(|| {
                            let value = self.__wasm_value.get_interned_obj_prop(interned_string_id);
                            shopify_function::wasm_api::Deserialize::deserialize(&value).unwrap()
                        });
                        let value_ref = &value;
                        #properly_referenced_value
                    }
                }
            })
            .collect();

        let accessor_impl = parse_quote! {
            impl #name_ident {
                #(#accessors)*
            }
        };

        vec![deserialize_impl, accessor_impl]
    }

    fn additional_impls_for_executable_enum(
        &self,
        executable_enum: &bluejay_typegen_codegen::ExecutableEnum,
    ) -> Vec<syn::ItemImpl> {
        let name_ident = names::type_ident(executable_enum.parent_name());

        let match_arms: Vec<syn::Arm> = executable_enum
            .variants()
            .iter()
            .map(|variant| {
                let variant_name_ident = names::enum_variant_ident(variant.parent_name());
                let variant_name_lit_str = syn::LitStr::new(variant.parent_name(), Span::mixed_site());

                parse_quote! {
                    #variant_name_lit_str => shopify_function::wasm_api::Deserialize::deserialize(value).map(Self::#variant_name_ident),
                }
            }).collect();

        vec![parse_quote! {
            impl shopify_function::wasm_api::Deserialize for #name_ident {
                fn deserialize(value: &shopify_function::wasm_api::Value) -> ::std::result::Result<Self, shopify_function::wasm_api::read::Error> {
                    let typename = value.get_obj_prop("__typename");
                    let typename_str: ::std::string::String = shopify_function::wasm_api::Deserialize::deserialize(&typename)?;

                    match typename_str.as_str() {
                        #(#match_arms)*
                        _ => ::std::result::Result::Ok(Self::Other),
                    }
                }
            }
        }]
    }

    fn additional_impls_for_enum(
        &self,
        enum_type_definition: &impl EnumTypeDefinition,
    ) -> Vec<syn::ItemImpl> {
        let name_ident = names::type_ident(enum_type_definition.name());

        let from_str_match_arms: Vec<syn::Arm> = enum_type_definition
            .enum_value_definitions()
            .iter()
            .map(|evd| {
                let variant_name_ident = names::enum_variant_ident(evd.name());
                let variant_name_lit_str = syn::LitStr::new(evd.name(), Span::mixed_site());

                parse_quote! {
                    #variant_name_lit_str => Self::#variant_name_ident,
                }
            })
            .collect();

        let as_str_match_arms: Vec<syn::Arm> = enum_type_definition
            .enum_value_definitions()
            .iter()
            .map(|evd| {
                let variant_name_ident = names::enum_variant_ident(evd.name());
                let variant_name_lit_str = syn::LitStr::new(evd.name(), Span::mixed_site());

                parse_quote! {
                    Self::#variant_name_ident => #variant_name_lit_str,
                }
            })
            .collect();

        let non_trait_method_impls = parse_quote! {
            impl #name_ident {
                pub fn from_str(s: &str) -> Self {
                    match s {
                        #(#from_str_match_arms)*
                        _ => Self::Other,
                    }
                }

                fn as_str(&self) -> &::std::primitive::str {
                    match self {
                        #(#as_str_match_arms)*
                        Self::Other => panic!("Cannot serialize `Other` variant"),
                    }
                }
            }
        };

        let serialize_impl = parse_quote! {
            impl shopify_function::wasm_api::Serialize for #name_ident {
                fn serialize(&self, context: &mut shopify_function::wasm_api::Context) -> ::std::result::Result<(), shopify_function::wasm_api::write::Error> {
                    let str_value = self.as_str();
                    context.write_utf8_str(str_value)
                }
            }
        };

        let deserialize_impl = parse_quote! {
            impl shopify_function::wasm_api::Deserialize for #name_ident {
                fn deserialize(value: &shopify_function::wasm_api::Value) -> ::std::result::Result<Self, shopify_function::wasm_api::read::Error> {
                    let str_value: ::std::string::String = shopify_function::wasm_api::Deserialize::deserialize(value)?;

                    ::std::result::Result::Ok(Self::from_str(&str_value))
                }
            }
        };

        let display_impl = parse_quote! {
            impl std::fmt::Display for #name_ident {
                fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                    write!(f, "{}", self.as_str())
                }
            }
        };

        vec![
            non_trait_method_impls,
            serialize_impl,
            deserialize_impl,
            display_impl,
        ]
    }

    fn additional_impls_for_input_object(
        &self,
        #[allow(unused_variables)] input_object_type_definition: &impl InputObjectTypeDefinition,
    ) -> Vec<syn::ItemImpl> {
        let name_ident = names::type_ident(input_object_type_definition.name());

        let field_statements: Vec<syn::Stmt> = input_object_type_definition
            .input_field_definitions()
            .iter()
            .flat_map(|ivd| {
                let field_name_ident = names::field_ident(ivd.name());
                let field_name_lit_str = syn::LitStr::new(ivd.name(), Span::mixed_site());

                vec![
                    parse_quote! {
                        context.write_utf8_str(#field_name_lit_str)?;
                    },
                    parse_quote! {
                        self.#field_name_ident.serialize(context)?;
                    },
                ]
            })
            .collect();

        let num_fields = input_object_type_definition.input_field_definitions().len();

        let serialize_impl = parse_quote! {
            impl shopify_function::wasm_api::Serialize for #name_ident {
                fn serialize(&self, context: &mut shopify_function::wasm_api::Context) -> ::std::result::Result<(), shopify_function::wasm_api::write::Error> {
                    context.write_object(
                        |context| {
                            #(#field_statements)*
                            ::std::result::Result::Ok(())
                        },
                        #num_fields,
                    )
                }
            }
        };

        let field_values: Vec<syn::FieldValue> = input_object_type_definition
            .input_field_definitions()
            .iter()
            .map(|ivd| {
                let field_name_ident = names::field_ident(ivd.name());
                let field_name_lit_str = syn::LitStr::new(ivd.name(), Span::mixed_site());
                parse_quote! { #field_name_ident: shopify_function::wasm_api::Deserialize::deserialize(&value.get_obj_prop(#field_name_lit_str))? }
            })
            .collect();

        let deserialize_impl = parse_quote! {
            impl shopify_function::wasm_api::Deserialize for #name_ident {
                fn deserialize(value: &shopify_function::wasm_api::Value) -> ::std::result::Result<Self, shopify_function::wasm_api::read::Error> {
                    ::std::result::Result::Ok(Self {
                        #(#field_values),*
                    })
                }
            }
        };

        vec![serialize_impl, deserialize_impl]
    }

    fn additional_impls_for_one_of_input_object(
        &self,
        input_object_type_definition: &impl InputObjectTypeDefinition,
    ) -> Vec<syn::ItemImpl> {
        let name_ident = names::type_ident(input_object_type_definition.name());

        let match_arms: Vec<syn::Arm> = input_object_type_definition
            .input_field_definitions()
            .iter()
            .map(|ivd| {
                let variant_ident = names::enum_variant_ident(ivd.name());
                let field_name_lit_str = syn::LitStr::new(ivd.name(), Span::mixed_site());

                parse_quote! {
                    Self::#variant_ident(value) => {
                        context.write_utf8_str(#field_name_lit_str)?;
                        shopify_function::wasm_api::Serialize::serialize(value, context)?;
                    }
                }
            })
            .collect();

        let serialize_impl = parse_quote! {
            impl shopify_function::wasm_api::Serialize for #name_ident {
                fn serialize(&self, context: &mut shopify_function::wasm_api::Context) -> ::std::result::Result<(), shopify_function::wasm_api::write::Error> {
                    context.write_object(|context| {
                        match self {
                            #(#match_arms)*
                        }
                        ::std::result::Result::Ok(())
                    }, 1)
                }
            }
        };

        let deserialize_match_arms: Vec<syn::Arm> = input_object_type_definition
            .input_field_definitions()
            .iter()
            .map(|ivd| {
                let field_name_lit_str = syn::LitStr::new(ivd.name(), Span::mixed_site());
                let variant_ident = names::enum_variant_ident(ivd.name());

                parse_quote! {
                    #field_name_lit_str => {
                        let value = shopify_function::wasm_api::Deserialize::deserialize(&field_value)?;
                        ::std::result::Result::Ok(Self::#variant_ident(value))
                    }
                }
            })
            .collect();

        let deserialize_impl = parse_quote! {
            impl shopify_function::wasm_api::Deserialize for #name_ident {
                fn deserialize(value: &shopify_function::wasm_api::Value) -> ::std::result::Result<Self, shopify_function::wasm_api::read::Error> {
                    let ::std::option::Option::Some(obj_len) = value.obj_len() else {
                        return ::std::result::Result::Err(shopify_function::wasm_api::read::Error::InvalidType);
                    };

                    if obj_len != 1 {
                        return ::std::result::Result::Err(shopify_function::wasm_api::read::Error::InvalidType);
                    }

                    let ::std::option::Option::Some(field_name) = value.get_obj_key_at_index(0) else {
                        return ::std::result::Result::Err(shopify_function::wasm_api::read::Error::InvalidType);
                    };
                    let field_value = value.get_at_index(0);

                    match field_name.as_str() {
                        #(#deserialize_match_arms)*
                        _ => ::std::result::Result::Err(shopify_function::wasm_api::read::Error::InvalidType),
                    }
                }
            }
        };

        vec![serialize_impl, deserialize_impl]
    }

    fn attributes_for_enum(
        &self,
        _enum_type_definition: &impl EnumTypeDefinition,
    ) -> Vec<syn::Attribute> {
        vec![
            parse_quote! { #[derive(::std::fmt::Debug, ::std::cmp::PartialEq, ::std::clone::Clone, ::std::marker::Copy)] },
        ]
    }

    fn attributes_for_input_object(
        &self,
        _input_object_type_definition: &impl InputObjectTypeDefinition,
    ) -> Vec<syn::Attribute> {
        vec![
            parse_quote! { #[derive(::std::fmt::Debug, ::std::cmp::PartialEq, ::std::clone::Clone)] },
        ]
    }

    fn attributes_for_one_of_input_object(
        &self,
        _input_object_type_definition: &impl InputObjectTypeDefinition,
    ) -> Vec<syn::Attribute> {
        vec![
            parse_quote! { #[derive(::std::fmt::Debug, ::std::cmp::PartialEq, ::std::clone::Clone)] },
        ]
    }
}

impl ShopifyFunctionCodeGenerator {
    fn type_for_field(
        executable_struct: &ExecutableStruct,
        r#type: &WrappedExecutableType,
        reference: bool,
    ) -> syn::Type {
        match r#type {
            WrappedExecutableType::Base(base) => {
                let base_type = executable_struct.compute_base_type(base);
                if reference {
                    parse_quote! { &#base_type }
                } else {
                    base_type
                }
            }
            WrappedExecutableType::Optional(inner) => {
                let inner_type = Self::type_for_field(executable_struct, inner, reference);
                parse_quote! { ::std::option::Option<#inner_type> }
            }
            WrappedExecutableType::Vec(inner) => {
                let inner_type = Self::type_for_field(executable_struct, inner, false);
                if reference {
                    parse_quote! { &[#inner_type] }
                } else {
                    parse_quote! { ::std::vec::Vec<#inner_type> }
                }
            }
        }
    }

    fn reference_variable_for_type(
        r#type: &WrappedExecutableType,
        variable: &syn::Ident,
    ) -> syn::Expr {
        match r#type {
            WrappedExecutableType::Base(_) => {
                parse_quote! { #variable }
            }
            WrappedExecutableType::Vec(_) => {
                parse_quote! { #variable.as_slice()}
            }
            WrappedExecutableType::Optional(inner) => {
                let inner_variable = format_ident!("v_inner");
                let inner_reference = Self::reference_variable_for_type(inner, &inner_variable);
                parse_quote! { ::std::option::Option::as_ref(#variable).map(|#inner_variable| #inner_reference) }
            }
        }
    }
}

/// Derives the `Deserialize` trait for structs to deserialize values from shopify_function_wasm_api::Value.
///
/// The derive macro supports the following attributes:
///
/// - `#[shopify_function(rename_all = "camelCase")]` - Converts field names from snake_case in Rust
///   to the specified case style ("camelCase", "snake_case", or "kebab-case") when deserializing.
///
/// - `#[shopify_function(default)]` - When applied to a field, uses the `Default` implementation for
///   that field's type if either:
///   1. The field's value is explicitly `null` in the JSON
///   2. The field is missing entirely from the JSON object
///
/// - `#[shopify_function(rename = "custom_name")]` - When applied to a field, uses the specified
///   custom name for deserialization instead of the field's Rust name. This takes precedence over
///   any struct-level `rename_all` attribute.
///
/// This is similar to serde's `#[serde(default)]` attribute, allowing structs to handle missing or null
/// fields gracefully by using their default values instead of returning an error.
///
/// Note: Fields that use `#[shopify_function(default)]` must be a type that implements the `Default` trait.
#[proc_macro_derive(Deserialize, attributes(shopify_function))]
pub fn derive_deserialize(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = syn::parse_macro_input!(input as syn::DeriveInput);

    derive_deserialize_for_derive_input(&input)
        .map(|impl_item| impl_item.to_token_stream().into())
        .unwrap_or_else(|error| error.to_compile_error().into())
}

#[derive(Default)]
struct FieldAttributes {
    rename: Option<String>,
    has_default: bool,
}

fn parse_field_attributes(field: &syn::Field) -> syn::Result<FieldAttributes> {
    let mut attributes = FieldAttributes::default();

    for attr in field.attrs.iter() {
        if attr.path().is_ident("shopify_function") {
            attr.parse_nested_meta(|meta| {
                if meta.path.is_ident("rename") {
                    attributes.rename = Some(meta.value()?.parse::<syn::LitStr>()?.value());
                    Ok(())
                } else if meta.path.is_ident("default") {
                    attributes.has_default = true;
                    Ok(())
                } else {
                    Err(meta.error("unrecognized field attribute"))
                }
            })?;
        }
    }

    Ok(attributes)
}

fn derive_deserialize_for_derive_input(input: &syn::DeriveInput) -> syn::Result<syn::ItemImpl> {
    match &input.data {
        syn::Data::Struct(data) => match &data.fields {
            syn::Fields::Named(fields) => {
                let name_ident = &input.ident;

                let mut rename_all: Option<syn::LitStr> = None;

                for attr in input.attrs.iter() {
                    if attr.path().is_ident("shopify_function") {
                        attr.parse_nested_meta(|meta| {
                            if meta.path.is_ident("rename_all") {
                                rename_all = Some(meta.value()?.parse()?);
                                Ok(())
                            } else {
                                Err(meta.error("unrecognized repr"))
                            }
                        })?;
                    }
                }

                let case_style = match rename_all {
                    Some(rename_all) => match rename_all.value().as_str() {
                        "camelCase" => Some(Case::Camel),
                        "snake_case" => Some(Case::Snake),
                        "kebab-case" => Some(Case::Kebab),
                        _ => {
                            return Err(syn::Error::new_spanned(
                                rename_all,
                                "unrecognized rename_all",
                            ))
                        }
                    },
                    None => None,
                };

                let field_values: Vec<syn::FieldValue> = fields
                    .named
                    .iter()
                    .map(|field| {
                        let field_name_ident = field.ident.as_ref().expect("Named fields must have identifiers");

                        let field_attrs = parse_field_attributes(field)?;

                        let field_name_str = match field_attrs.rename {
                            Some(custom_name) => custom_name,
                            None => {
                                // Fall back to rename_all case transformation or original name
                                case_style.map_or_else(
                                    || field_name_ident.to_string(),
                                    |case_style| field_name_ident.to_string().to_case(case_style)
                                )
                            }
                        };

                        let field_name_lit_str = syn::LitStr::new(field_name_str.as_str(), Span::mixed_site());

                        if field_attrs.has_default {
                            // For fields with default attribute, check if value is null or missing
                            // This will use the Default implementation for the field type when either:
                            // 1. The field is explicitly null in the JSON (we get NanBox::null())
                            // 2. The field is missing in the JSON (get_obj_prop returns a null value)
                            Ok(parse_quote! {
                                #field_name_ident: {
                                    let prop = value.get_obj_prop(#field_name_lit_str);
                                    if prop.is_null() {
                                        ::std::default::Default::default()
                                    } else {
                                        shopify_function::wasm_api::Deserialize::deserialize(&prop)?
                                    }
                                }
                            })
                        } else {
                            // For fields without default, use normal deserialization
                            Ok(parse_quote! {
                                #field_name_ident: shopify_function::wasm_api::Deserialize::deserialize(&value.get_obj_prop(#field_name_lit_str))?
                            })
                        }
                    })
                    .collect::<syn::Result<Vec<_>>>()?;

                let deserialize_impl = parse_quote! {
                    impl shopify_function::wasm_api::Deserialize for #name_ident {
                        fn deserialize(value: &shopify_function::wasm_api::Value) -> ::std::result::Result<Self, shopify_function::wasm_api::read::Error> {
                            ::std::result::Result::Ok(Self {
                                #(#field_values),*
                            })
                        }
                    }
                };

                Ok(deserialize_impl)
            }
            syn::Fields::Unnamed(_) | syn::Fields::Unit => Err(syn::Error::new_spanned(
                input,
                "Structs must have named fields to derive `Deserialize`",
            )),
        },
        syn::Data::Enum(_) => Err(syn::Error::new_spanned(
            input,
            "Enum types are not supported for deriving `Deserialize`",
        )),
        syn::Data::Union(_) => Err(syn::Error::new_spanned(
            input,
            "Union types are not supported for deriving `Deserialize`",
        )),
    }
}
