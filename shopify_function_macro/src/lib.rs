use convert_case::{Case, Casing};
use graphql_client_codegen::{
    generate_module_token_stream_from_string, CodegenMode, GraphQLClientCodegenOptions,
};
use std::path::Path;

use proc_macro2::{Ident, Span, TokenStream};
use quote::{quote, ToTokens};
use syn::{
    self, parse::Parse, parse::ParseStream, parse_macro_input, Expr, ExprArray, FnArg, LitStr,
    Token,
};

#[derive(Clone, Default)]
struct ShopifyFunctionArgs {
    input_stream: Option<Expr>,
    output_stream: Option<Expr>,
}

impl ShopifyFunctionArgs {
    fn parse_expression<T: syn::parse::Parse>(input: &ParseStream<'_>) -> syn::Result<Expr> {
        input.parse::<T>()?;
        input.parse::<Token![=]>()?;
        let value: Expr = input.parse()?;
        Ok(value)
    }
}

impl Parse for ShopifyFunctionArgs {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let mut args = Self::default();
        while !input.is_empty() {
            let lookahead = input.lookahead1();
            if lookahead.peek(kw::input_stream) {
                args.input_stream = Some(Self::parse_expression::<kw::input_stream>(&input)?);
            } else if lookahead.peek(kw::output_stream) {
                args.output_stream = Some(Self::parse_expression::<kw::output_stream>(&input)?);
            } else {
                // Ignore unknown tokens
                let _ = input.parse::<proc_macro2::TokenTree>();
            }
        }
        Ok(args)
    }
}

/// Marks a function as a Shopify Function entry point.
///
/// This attribute marks the following function as the entry point
/// for a Shopify Function. A Shopify Function takes exactly one
/// parameter of type `input::ResponseData`, and returns a
/// `Result<output::FunctionResult>`. Both of these types are generated
/// at build time from the Shopify's GraphQL schema. Take a look at the
/// [`macro@generate_types`] macro for details on those types.
///
/// ```ignore
/// #[shopify_function]
/// fn function(input: input::ResponseData) -> Result<output::FunctionResult> {
///     /* ... */
/// }
/// ```
///
/// By default, the function input is read from stdin and the result
/// is outputted to stdout. To override this, optional `input_stream`
/// and `output_stream` parameters can be set. These parameters must
/// implement the std::io::Read and std::io::Write traits respectively.
///
/// ```ignore
/// #[shopify_function(input_stream = MyInputStream, output_stream = MyOutputStream)]
/// fn function(input: input::ResponseData) -> Result<output::FunctionResult> {
///     /* ... */
/// }
/// ```
#[proc_macro_attribute]
pub fn shopify_function(
    attr: proc_macro::TokenStream,
    item: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    let ast = parse_macro_input!(item as syn::ItemFn);
    let args = parse_macro_input!(attr as ShopifyFunctionArgs);

    let name = &ast.sig.ident;
    if ast.sig.inputs.len() != 1 {
        return quote! {compile_error!("Shopify functions need exactly one input parameter");}
            .into();
    }

    let input_type = match &ast.sig.inputs.first().unwrap() {
        FnArg::Typed(input) => input.ty.as_ref(),
        FnArg::Receiver(_) => {
            return quote! {compile_error!("Shopify functions can’t have a receiver");}.into()
        }
    };

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

    let gen = quote! {
        fn main() -> ::shopify_function::Result<()> {
            let mut string = String::new();
            std::io::Read::read_to_string(&mut #input_stream, &mut string)?;
            let input: #input_type = serde_json::from_str(&string)?;
            let mut out = #output_stream;
            let result = #name(input)?;
            let serialized = serde_json::to_vec(&result)?;
            std::io::Write::write_all(&mut out, serialized.as_slice())?;
            Ok(())
        }
        #ast
    };

    gen.into()
}

#[derive(Clone, Default)]
struct ShopifyFunctionTargetArgs {
    target: Option<LitStr>,
    module_name: Option<LitStr>,
    query_path: Option<LitStr>,
    schema_path: Option<LitStr>,
    input_stream: Option<Expr>,
    output_stream: Option<Expr>,
    extern_enums: Option<ExprArray>,
}

impl ShopifyFunctionTargetArgs {
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

impl Parse for ShopifyFunctionTargetArgs {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let mut args = Self::default();
        while !input.is_empty() {
            let lookahead = input.lookahead1();
            if lookahead.peek(kw::target) {
                args.target = Some(Self::parse::<kw::target, LitStr>(&input)?);
            } else if lookahead.peek(kw::module_name) {
                args.module_name = Some(Self::parse::<kw::module_name, LitStr>(&input)?);
            } else if lookahead.peek(kw::query_path) {
                args.query_path = Some(Self::parse::<kw::query_path, LitStr>(&input)?);
            } else if lookahead.peek(kw::schema_path) {
                args.schema_path = Some(Self::parse::<kw::schema_path, LitStr>(&input)?);
            } else if lookahead.peek(kw::input_stream) {
                args.input_stream = Some(Self::parse::<kw::input_stream, Expr>(&input)?);
            } else if lookahead.peek(kw::output_stream) {
                args.output_stream = Some(Self::parse::<kw::output_stream, Expr>(&input)?);
            } else if lookahead.peek(kw::extern_enums) {
                args.extern_enums = Some(Self::parse::<kw::extern_enums, ExprArray>(&input)?);
            } else {
                return Err(lookahead.error());
            }
        }
        Ok(args)
    }
}

#[derive(Clone, Default)]
struct GenerateTypeArgs {
    query_path: Option<LitStr>,
    schema_path: Option<LitStr>,
    input_stream: Option<Expr>,
    output_stream: Option<Expr>,
    extern_enums: Option<ExprArray>,
}

impl GenerateTypeArgs {
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

impl Parse for GenerateTypeArgs {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let mut args = Self::default();
        while !input.is_empty() {
            let lookahead = input.lookahead1();
            if lookahead.peek(kw::query_path) {
                args.query_path = Some(Self::parse::<kw::query_path, LitStr>(&input)?);
            } else if lookahead.peek(kw::schema_path) {
                args.schema_path = Some(Self::parse::<kw::schema_path, LitStr>(&input)?);
            } else if lookahead.peek(kw::input_stream) {
                args.input_stream = Some(Self::parse::<kw::input_stream, Expr>(&input)?);
            } else if lookahead.peek(kw::output_stream) {
                args.output_stream = Some(Self::parse::<kw::output_stream, Expr>(&input)?);
            } else if lookahead.peek(kw::extern_enums) {
                args.extern_enums = Some(Self::parse::<kw::extern_enums, ExprArray>(&input)?);
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
///
///
/// The macro takes the following parameters:
/// - `query_path`: A path to a GraphQL query, whose result will be used
///    as the input for the function invocation. The query MUST be named "Input".
/// - `schema_path`: A path to Shopify's GraphQL schema definition. Use the CLI
///   to download a fresh copy.
/// - `target` (optional): The API-specific handle for the target if the function name does not match the target handle as `snake_case`
/// - `module_name` (optional): The name of the generated module.
///   - default: The target handle as `snake_case`
/// - `extern_enums` (optional): A list of Enums for which an external type should be used.
///   For those, code generation will be skipped. This is useful for large enums
///   which can increase binary size, or for enums shared between multiple targets.
///   Example: `extern_enums = ["LanguageCode"]`
///    - default: `["LanguageCode", "CountryCode", "CurrencyCode"]`
#[proc_macro_attribute]
pub fn shopify_function_target(
    attr: proc_macro::TokenStream,
    item: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    let ast = parse_macro_input!(item as syn::ItemFn);
    let args = parse_macro_input!(attr as ShopifyFunctionTargetArgs);

    let function_name = &ast.sig.ident;
    let function_name_string = function_name.to_string();
    let target_handle_string = args.target.map_or(function_name_string.clone(), |target| {
        target
            .value()
            .split('.')
            .collect::<Vec<&str>>()
            .last()
            .unwrap()
            .to_case(Case::Snake)
    });
    let module_name = args.module_name.map_or(
        Ident::new(&target_handle_string, Span::mixed_site()),
        |module_name| Ident::new(module_name.value().as_str(), Span::mixed_site()),
    );

    let query_path = args
        .query_path
        .expect("No value given for query_path")
        .value();
    let schema_path = args
        .schema_path
        .expect("No value given for schema_path")
        .value();
    let extern_enums = args.extern_enums.as_ref().map(extract_extern_enums);

    let input_struct = generate_input_struct(
        query_path.as_str(),
        schema_path.as_str(),
        extern_enums.as_deref(),
    );

    if let Err(error) = extract_shopify_function_return_type(&ast) {
        return error.to_compile_error().into();
    }
    let output_result_type = extract_shopify_function_return_type(&ast)
        .unwrap()
        .to_token_stream()
        .to_string();
    let output_query = format!(
        "mutation Output($result: {}!) {{\n    {}(result: $result)\n}}\n",
        output_result_type,
        &target_handle_string.to_case(Case::Camel)
    );
    let output_struct =
        generate_output_struct(&output_query, schema_path.as_str(), extern_enums.as_deref());

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

    quote! {
        pub mod #module_name {
            use super::*;
            use std::io::Write;

            #input_struct
            #output_struct

            #[shopify_function(
                input_stream = #input_stream,
                output_stream = #output_stream
            )]
            pub #ast

            #[export_name = #function_name_string]
            pub extern "C" fn export() {
                main().unwrap();
                #output_stream.flush().unwrap();
            }
        }
        pub use #module_name::#function_name;
    }
    .into()
}

/// Generate the types to interact with Shopify's API.
///
/// The macro generates two inline modules: `input` and `output`. The
/// modules generate Rust types from the GraphQL schema file for the Function input
/// and output respectively.
///
/// The macro takes the following parameters:
/// - `query_path`: A path to a GraphQL query, whose result will be used
///    as the input for the function invocation. The query MUST be named "Input".
/// - `schema_path`: A path to Shopify's GraphQL schema definition. Use the CLI
///   to download a fresh copy.
/// - `extern_enums` (optional): A list of Enums for which an external type should be used.
///   For those, code generation will be skipped. This is useful for large enums
///   which can increase binary size, or for enums shared between multiple targets.
///   Example: `extern_enums = ["LanguageCode"]`
///    - default: `["LanguageCode", "CountryCode", "CurrencyCode"]`
#[proc_macro]
pub fn generate_types(attr: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let args = parse_macro_input!(attr as GenerateTypeArgs);

    let query_path = args
        .query_path
        .expect("No value given for query_path")
        .value();
    let schema_path = args
        .schema_path
        .expect("No value given for schema_path")
        .value();
    let extern_enums = args.extern_enums.as_ref().map(extract_extern_enums);
    let input_struct = generate_input_struct(
        query_path.as_str(),
        schema_path.as_str(),
        extern_enums.as_deref(),
    );
    let output_query =
        "mutation Output($result: FunctionResult!) {\n    handleResult(result: $result)\n}\n";
    let output_struct = generate_output_struct(output_query, &schema_path, extern_enums.as_deref());

    quote! {
        #input_struct
        #output_struct
    }
    .into()
}

const DEFAULT_EXTERN_ENUMS: &[&str] = &["LanguageCode", "CountryCode", "CurrencyCode"];

fn generate_input_struct(
    query_path: &str,
    schema_path: &str,
    extern_enums: Option<&[String]>,
) -> TokenStream {
    let extern_enums = extern_enums
        .map(|e| e.to_owned())
        .unwrap_or_else(|| DEFAULT_EXTERN_ENUMS.iter().map(|e| e.to_string()).collect());

    quote! {
        #[derive(graphql_client::GraphQLQuery, Clone, Debug, serde::Deserialize, PartialEq)]
        #[graphql(
            query_path = #query_path,
            schema_path = #schema_path,
            response_derives = "Clone,Debug,PartialEq,Deserialize,Serialize",
            variables_derives = "Clone,Debug,PartialEq,Deserialize",
            extern_enums(#(#extern_enums),*),
            skip_serializing_none
        )]
        pub struct Input;
    }
}

fn graphql_codegen_options(
    operation_name: String,
    extern_enums: Option<&[String]>,
) -> GraphQLClientCodegenOptions {
    let mut options = GraphQLClientCodegenOptions::new(CodegenMode::Derive);
    options.set_operation_name(operation_name);
    options.set_response_derives("Clone,Debug,PartialEq,Deserialize,Serialize".to_string());
    options.set_variables_derives("Clone,Debug,PartialEq,Deserialize".to_string());
    options.set_skip_serializing_none(true);
    if let Some(extern_enums) = extern_enums {
        options.set_extern_enums(extern_enums.to_vec());
    }

    options
}

fn generate_output_struct(
    query: &str,
    schema_path: &str,
    extern_enums: Option<&[String]>,
) -> proc_macro2::TokenStream {
    let options = graphql_codegen_options("Output".to_string(), extern_enums);
    let cargo_manifest_dir =
        std::env::var("CARGO_MANIFEST_DIR").expect("Error reading CARGO_MANIFEST_DIR from env");
    let schema_path = Path::new(&cargo_manifest_dir).join(schema_path);
    let token_stream = generate_module_token_stream_from_string(query, &schema_path, options)
        .expect("Error generating Output struct");

    quote! {
        #token_stream
        pub struct Output;
    }
}

fn extract_extern_enums(extern_enums: &ExprArray) -> Vec<String> {
    let extern_enum_error_msg = r#"The `extern_enums` attribute expects comma separated string literals\n\n= help: use `extern_enums = ["Enum1", "Enum2"]`"#;
    extern_enums
        .elems
        .iter()
        .map(|expr| {
            let value = match expr {
                Expr::Lit(lit) => lit.lit.clone(),
                _ => panic!("{}", extern_enum_error_msg),
            };
            match value {
                syn::Lit::Str(lit) => lit.value(),
                _ => panic!("{}", extern_enum_error_msg),
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {}

mod kw {
    syn::custom_keyword!(target);
    syn::custom_keyword!(module_name);
    syn::custom_keyword!(query_path);
    syn::custom_keyword!(schema_path);
    syn::custom_keyword!(input_stream);
    syn::custom_keyword!(output_stream);
    syn::custom_keyword!(extern_enums);
}
