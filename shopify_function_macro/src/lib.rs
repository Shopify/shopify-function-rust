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
use proc_macro2::Span;
use quote::{format_ident, quote, ToTokens};
use syn::{
    parse::{Parse, ParseStream},
    parse_macro_input, parse_quote, Expr, FnArg, Token,
};

#[derive(Clone, Default)]
struct ShopifyFunctionArgs {
    input_stream: Option<Expr>,
    output_stream: Option<Expr>,
}

impl ShopifyFunctionArgs {
    fn parse<K: syn::parse::Parse, V: syn::parse::Parse>(
        input: &ParseStream<'_>,
    ) -> syn::Result<V> {
        input.parse::<K>()?;
        input.parse::<Token![=]>()?;
        let value: V = input.parse()?;
        if input.lookahead1().peek(Token![,]) {
            input.parse::<Token![,]>()?;
        }
        Ok(value)
    }
}

impl Parse for ShopifyFunctionArgs {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let mut args = Self::default();
        while !input.is_empty() {
            let lookahead = input.lookahead1();
            if lookahead.peek(kw::input_stream) {
                args.input_stream = Some(Self::parse::<kw::input_stream, Expr>(&input)?);
            } else if lookahead.peek(kw::output_stream) {
                args.output_stream = Some(Self::parse::<kw::output_stream, Expr>(&input)?);
            } else {
                return Err(lookahead.error());
            }
        }
        Ok(args)
    }
}

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

/// Generates code for a Function using an explicitly-named target. This will:
/// - Generate a module to host the generated types.
/// - Generate types based on the GraphQL schema for the Function input and output.
/// - Define a wrapper function that's exported to Wasm. The wrapper handles
///   decoding the input from STDIN, and encoding the output to STDOUT.
#[proc_macro_attribute]
pub fn shopify_function(
    attr: proc_macro::TokenStream,
    item: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    let ast = parse_macro_input!(item as syn::ItemFn);
    let args = parse_macro_input!(attr as ShopifyFunctionArgs);

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
            let root_value = context.input_get().unwrap();
            let mut input: #input_type = shopify_function::wasm_api::Deserialize::deserialize(&root_value).unwrap();
            let result = #function_name(input).unwrap();
            shopify_function::wasm_api::Serialize::serialize(&result, &mut context).unwrap();
            context.finalize_output().unwrap();
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
/// By default, will implement `PartialEq`, `Eq`, `Clone`, and `Debug` for all types. Will implement `Copy` for enums.
/// For types corresponding to values returned from queries,  the relevant deserialization trait for the selected codec
/// is implemented (e.g. `serde::Deserialize` in the case of `serde`). For types that would
/// be arguments to a query, the relevant serialization trait for the selected codec is implemented
/// (e.g. `serde::Serialize` for the `serde` codec).
///
/// ### Usage
///
/// Must be used with a module. Inside the module, type aliases must be defined for any custom scalars in the schema.
/// To use a query, define a module within the aforementioned module, and annotate it with
/// `#[query("path/to/query.graphql")]`, where the argument is a string literal path to the query document, or the
/// query contents enclosed in square brackets.
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

    // TODO: disallow `borrow` value of `true` for `input`

    if input.enums_as_str.is_empty() {
        let enums_as_str = DEFAULT_EXTERN_ENUMS
            .iter()
            .map(|enum_name| syn::LitStr::new(enum_name, Span::mixed_site()))
            .collect::<Vec<_>>();
        input.enums_as_str = syn::parse_quote! { #(#enums_as_str),* };
    }

    let string_known_custom_scalar_type = KnownCustomScalarType {
        type_for_borrowed: Some(syn::parse_quote! { ::std::borrow::Cow<'a, str> }),
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
                type_for_owned: syn::parse_quote! { ::shopify_function::serde_json::Value },
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
        _executable_struct: &bluejay_typegen_codegen::ExecutableStruct,
    ) -> syn::Fields {
        let fields_named: syn::FieldsNamed = parse_quote! {
            {
                value: shopify_function::wasm_api::Value,
            }
        };
        fields_named.into()
    }

    // fn field_accessor_block(
    //     &self,
    //     _executable_struct: &bluejay_typegen_codegen::ExecutableStruct,
    //     field: &bluejay_typegen_codegen::ExecutableField,
    // ) -> syn::Block {
    //     let field_name_ident = names::field_ident(field.graphql_name());
    //     let field_name_lit_str = syn::LitStr::new(field.graphql_name(), Span::mixed_site());

    //     let properly_referenced_value =
    //         Self::reference_variable_for_type(field.r#type(), &format_ident!("value"));

    //     parse_quote! {
    //         {
    //             let value = self.#field_name_ident.get_or_init(|| {
    //                 let value = self.__wasm_value.get_obj_prop(#field_name_lit_str);
    //                 shopify_function::wasm_api::Deserialize::deserialize(&value).unwrap()
    //             });
    //             #properly_referenced_value
    //         }
    //     }
    // }

    fn additional_impls_for_executable_struct(
        &self,
        executable_struct: &bluejay_typegen_codegen::ExecutableStruct,
    ) -> Vec<syn::ItemImpl> {
        let name_ident = names::type_ident(executable_struct.parent_name());

        let deserialize_impl = parse_quote! {
            impl shopify_function::wasm_api::Deserialize for #name_ident {
                fn deserialize(value: &shopify_function::wasm_api::Value) -> ::std::result::Result<Self, shopify_function::wasm_api::read::Error> {
                    Ok(Self {
                        value: *value,
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
                let field_type = Self::type_for_field(executable_struct, field.r#type());

                parse_quote! {
                    pub fn #field_name_ident(&self) -> #field_type {
                        let value = self.value.get_obj_prop(#field_name_lit_str);
                        shopify_function::wasm_api::Deserialize::deserialize(&value).unwrap()
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
                    let typename_str: String = shopify_function::wasm_api::Deserialize::deserialize(&typename)?;

                    match typename_str.as_str() {
                        #(#match_arms)*
                        _ => Ok(Self::Other),
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

        let match_arms: Vec<syn::Arm> = enum_type_definition
            .enum_value_definitions()
            .iter()
            .map(|evd| {
                let variant_name_ident = names::enum_variant_ident(evd.name());
                let variant_name_lit_str = syn::LitStr::new(evd.name(), Span::mixed_site());
                parse_quote! {
                    Self::#variant_name_ident => context.write_utf8_str(#variant_name_lit_str),
                }
            })
            .collect();

        let serialize_impl = parse_quote! {
            impl shopify_function::wasm_api::Serialize for #name_ident {
                fn serialize(&self, context: &mut shopify_function::wasm_api::Context) -> ::std::result::Result<(), shopify_function::wasm_api::write::Error> {
                    match self {
                        #(#match_arms)*
                        Self::Other => panic!("Cannot serialize `Other` variant"),
                    }
                }
            }
        };

        vec![serialize_impl]
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
                            Ok(())
                        },
                        #num_fields,
                    )
                }
            }
        };

        vec![serialize_impl]
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
                        Ok(())
                    }, 1)
                }
            }
        };

        vec![serialize_impl]
    }
}

impl ShopifyFunctionCodeGenerator {
    fn type_for_field(
        executable_struct: &ExecutableStruct,
        r#type: &WrappedExecutableType,
    ) -> syn::Type {
        match r#type {
            WrappedExecutableType::Base(base) => executable_struct.compute_base_type(base),
            WrappedExecutableType::Optional(inner) => {
                let inner_type = Self::type_for_field(executable_struct, inner);
                parse_quote! { ::std::option::Option<#inner_type> }
            }
            WrappedExecutableType::Vec(inner) => {
                let inner_type = Self::type_for_field(executable_struct, inner);
                parse_quote! { shopify_function::Iter<#inner_type> }
            }
        }
    }
}
