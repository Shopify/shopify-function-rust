use std::collections::HashMap;

use bluejay_typegen_codegen::{generate_schema, Input as BluejayInput, KnownCustomScalarType};

use proc_macro2::Span;
use quote::{format_ident, quote, ToTokens};
use syn::{
    self,
    parse::{Parse, ParseStream},
    parse_macro_input, Expr, FnArg, Token,
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

    let input_stream = args
        .input_stream
        .map_or(quote! { std::io::stdin() }, |stream| {
            stream.to_token_stream()
        });

    let output_stream = args
        .output_stream
        .map_or(quote! { std::io::stdout() }, |stream| {
            stream.to_token_stream()
        });

    let deserialize_line: syn::Stmt = syn::parse_quote! {
        let input: #input_type = shopify_function::serde_json::from_str(&string).unwrap();
    };

    let serialize_line: syn::Stmt = syn::parse_quote! {
        let serialized = shopify_function::serde_json::to_vec(&result).unwrap();
    };

    let write_line: syn::Stmt = syn::parse_quote! {
        std::io::Write::write_all(&mut out, serialized.as_slice()).unwrap();
    };

    quote! {
        #[export_name = #function_name_string]
        pub extern "C" fn #export_function_name() {
            let mut string = String::new();
            std::io::Read::read_to_string(&mut #input_stream, &mut string).unwrap();
            #deserialize_line
            let mut out = #output_stream;
            let result = #function_name(input).unwrap();
            #serialize_line
            #write_line
            std::io::Write::flush(&mut out).unwrap();
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

    if let Err(error) = generate_schema(input, &mut module, known_custom_scalar_types) {
        return error.to_compile_error().into();
    }

    module.to_token_stream().into()
}
