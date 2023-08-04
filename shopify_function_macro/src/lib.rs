use convert_case::{Case, Casing};
use std::io::Write;
use std::path::Path;

use proc_macro2::{Ident, Span, TokenStream, TokenTree};
use quote::{quote, ToTokens};
use syn::{self, parse::Parse, parse::ParseStream, parse_macro_input, Expr, FnArg, LitStr, Token};

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
        return Err(Error::new_spanned(&ast.sig, "Shopify Functions require an explicit return type"))
    };
    let Type::Path(path) = ty.as_ref() else {
        return Err(Error::new_spanned(&ast.sig, "Shopify Functions must return a Result"))
    };
    let result = path.path.segments.last().unwrap();
    if result.ident != "Result" {
        return Err(Error::new_spanned(
            result,
            "Shopify Functions must return a Result",
        ));
    }
    let PathArguments::AngleBracketed(generics) = &result.arguments else {
        return Err(Error::new_spanned(result, "Shopify Function Result is missing generic arguments"))
    };
    if generics.args.len() != 1 {
        return Err(Error::new_spanned(
            generics,
            "Shopify Function Result takes exactly one generic argument",
        ));
    }
    let GenericArgument::Type(ty) = generics.args.first().unwrap() else {
        return Err(Error::new_spanned(generics, "Shopify Function Result expects a type"))
    };
    let Type::Path(path) = ty else {
        return Err(Error::new_spanned(result, "Unexpected result type for Shopify Function Result"))
    };
    Ok(&path.path.segments.last().as_ref().unwrap().ident)
}

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

    let query_path = args.query_path.expect("No value given for query_path");
    let schema_path = args.schema_path.expect("No value given for schema_path");
    let output_query_file_name = format!(".{}{}", &target_handle_string, OUTPUT_QUERY_FILE_NAME);

    let input_struct = generate_struct(
        "Input",
        query_path.value().as_str(),
        schema_path.value().as_str(),
    );
    let output_struct = generate_struct(
        "Output",
        &output_query_file_name,
        schema_path.value().as_str(),
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

    write_output_query_file(&output_query_file_name, &output_query);

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

fn extract_attr(attrs: &TokenStream, attr: &str) -> String {
    let attrs: Vec<TokenTree> = attrs.clone().into_iter().collect();
    let attr_index = attrs
        .iter()
        .position(|item| match item {
            TokenTree::Ident(ident) => ident.to_string().as_str() == attr,
            _ => false,
        })
        .unwrap_or_else(|| panic!("No attribute with name {} found", attr));
    let value = attrs
        .get(attr_index + 2)
        .unwrap_or_else(|| panic!("No value given for {} attribute", attr))
        .to_string();
    value.as_str()[1..value.len() - 1].to_string()
}

const OUTPUT_QUERY_FILE_NAME: &str = ".output.graphql";

/// Generate the types to interact with Shopify's API.
///
/// The macro generates two inline modules: `input` and `output`. The
/// modules generate Rust types from the GraphQL schema file for the Function input
/// and output respectively.
///
/// The macro takes two parameters:
/// - `query_path`: A path to a GraphQL query, whose result will be used
///    as the input for the function invocation. The query MUST be named "Input".
/// - `schema_path`: A path to Shopify's GraphQL schema definition. You
///   can find it in the `example` folder of the repo, or use the CLI
///   to download a fresh copy (not implemented yet).
///
/// Note: This macro creates a file called `.output.graphql` in the root
/// directory of the project. It can be safely added to your `.gitignore`. We
/// hope we can avoid creating this file at some point in the future.
#[proc_macro]
pub fn generate_types(attr: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let params = TokenStream::from(attr);

    let query_path = extract_attr(&params, "query_path");
    let schema_path = extract_attr(&params, "schema_path");

    let input_struct = generate_struct("Input", &query_path, &schema_path);
    let output_struct = generate_struct("Output", OUTPUT_QUERY_FILE_NAME, &schema_path);
    let output_query =
        "mutation Output($result: FunctionResult!) {\n    handleResult(result: $result)\n}\n";

    write_output_query_file(OUTPUT_QUERY_FILE_NAME, output_query);

    quote! {
        #input_struct
        #output_struct
    }
    .into()
}

fn generate_struct(name: &str, query_path: &str, schema_path: &str) -> TokenStream {
    let name_ident = Ident::new(name, Span::mixed_site());

    quote! {
        #[derive(graphql_client::GraphQLQuery, Clone, Debug, serde::Deserialize, PartialEq)]
        #[graphql(
            query_path = #query_path,
            schema_path = #schema_path,
            response_derives = "Clone,Debug,PartialEq,Eq,Deserialize,Serialize",
            variables_derives = "Clone,Debug,PartialEq,Eq,Deserialize",
            skip_serializing_none
        )]
        pub struct #name_ident;
    }
}

fn write_output_query_file(output_query_file_name: &str, contents: &str) {
    let cargo_manifest_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap();
    let output_query_path = Path::new(&cargo_manifest_dir).join(output_query_file_name);
    std::fs::File::create(output_query_path)
        .expect("Could not create output query file")
        .write_all(contents.as_bytes())
        .unwrap_or_else(|_| panic!("Could not write to {}", output_query_file_name));
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
}
