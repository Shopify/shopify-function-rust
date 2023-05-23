use std::io::Write;
use std::path::Path;

use proc_macro2::TokenStream;
use proc_macro2::TokenTree;

use quote::{quote, ToTokens};
use syn::{self, parse_macro_input, parse::Parse, parse::ParseStream, parse_macro_input, Expr, FnArg, Token};

#[derive(Clone, Default, Debug)]
struct ShopifyFunctionArgs {
    input_stream: Option<Expr>,
    output_stream: Option<Expr>,
}

impl ShopifyFunctionArgs {
    fn parse_expression<T: syn::parse::Parse>(input: &ParseStream<'_>) -> syn::Result<Expr> {
        let _ = input.parse::<T>()?;
        let _ = input.parse::<Token![=]>()?;
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
    let ast: syn::ItemFn = syn::parse(item).unwrap();
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
            let mut serializer = serde_json::Serializer::new(&mut out);
            #name(input)?.serialize(&mut serializer)?;
            Ok(())
        }
        #ast
    };

    gen.into()
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

    let cargo_manifest_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap();
    let schema_path = extract_attr(&params, "schema_path");

    let mut output_query_path = Path::new(&cargo_manifest_dir).to_path_buf();
    output_query_path.push(OUTPUT_QUERY_FILE_NAME);
    std::fs::File::create(&output_query_path)
        .expect("Could not create output query file")
        .write_all(
            r"
                mutation Output($result: FunctionResult!) {
                    handleResult(result: $result)
                }
            "
            .as_bytes(),
        )
        .expect("Could not write to .output.query");

    quote! {
        #[derive(graphql_client::GraphQLQuery, Clone, Debug, serde::Deserialize, PartialEq)]
        #[serde(rename_all(deserialize = "camelCase"))]
        #[graphql(
            #params,
            response_derives = "Clone,Debug,PartialEq,Eq,Deserialize,OneOfDefault",
            variables_derives = "Clone,Debug,PartialEq,Eq,Deserialize,OneOfDefault",
            skip_serializing_none
        )]
        struct Input;

        #[derive(graphql_client::GraphQLQuery, Clone, Debug, serde::Deserialize, PartialEq)]
        #[graphql(
            query_path = #OUTPUT_QUERY_FILE_NAME,
            schema_path = #schema_path,
            response_derives = "Clone,Debug,PartialEq,Eq,Deserialize,OneOfDefault",
            variables_derives = "Clone,Debug,PartialEq,Eq,Deserialize,OneOfDefault",
            skip_serializing_none
        )]
        struct Output;
    }
    .into()
}

fn is_type(ty: &syn::Type, name: impl AsRef<str>) -> bool {
    if let syn::Type::Path(path) = ty {
        path.path.segments.len() == 1
            && path
                .path
                .segments
                .first()
                .unwrap()
                .ident
                .to_string()
                .as_str()
                == name.as_ref()
    } else {
        false
    }
}

#[proc_macro_derive(OneOfDefault)]
pub fn one_of_default_derive(item: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let parsed_item: syn::Item = parse_macro_input!(item);
    let syn::Item::Struct(parsed_struct) = parsed_item else { return quote!{}.into()};
    // If every field in the struct is an `Option`, it's the result
    // of a `@oneOf` directive.
    let is_oneof_struct = parsed_struct
        .fields
        .iter()
        .all(|field| is_type(&field.ty, "Option"));

    let derive_impl = if is_oneof_struct {
        let struct_name = parsed_struct.ident.clone();
        let field_inits: Vec<TokenStream> = parsed_struct
            .fields
            .iter()
            .map(|field| {
                let ident = &field.ident;
                quote! {
                    #ident: None
                }
            })
            .collect();
        quote! {
            impl ::core::default::Default for #struct_name {
                fn default() -> Self {
                    #struct_name {
                        #(#field_inits),*
                    }
                }
            }
        }
    } else {
        quote! {}
    };

    quote! {
        #derive_impl
    }
    .into()
}

#[cfg(test)]
mod tests {}

mod kw {
    syn::custom_keyword!(input_stream);
    syn::custom_keyword!(output_stream);
}
