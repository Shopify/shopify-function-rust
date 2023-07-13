use std::io::Write;
use std::path::Path;

use proc_macro2::Span;
use quote::{quote, ToTokens};
use syn::{
    self, parse::Parse, parse::ParseStream, parse_macro_input, Expr, FnArg, Ident, LitStr, Token,
};

#[derive(Clone, Default)]
struct ShopifyFunctionArgs {
    export: Option<Ident>,
    query: Option<Ident>,
    mutation: Option<Ident>,
    query_path: Option<LitStr>,
    schema_path: Option<LitStr>,
    input_stream: Option<Expr>,
    output_stream: Option<Expr>,
}

impl ShopifyFunctionArgs {
    fn parse_expression<K: syn::parse::Parse, V: syn::parse::Parse>(
        input: &ParseStream<'_>,
    ) -> syn::Result<V> {
        let _ = input.parse::<K>()?;
        let _ = input.parse::<Token![=]>()?;
        let value: V = input.parse()?;
        Ok(value)
    }
}

impl Parse for ShopifyFunctionArgs {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let mut args = Self::default();
        while !input.is_empty() {
            let lookahead = input.lookahead1();
            if lookahead.peek(kw::export) {
                args.export = Some(Self::parse_expression::<kw::export, Ident>(&input)?);
            } else if lookahead.peek(kw::query) {
                args.query = Some(Self::parse_expression::<kw::query, Ident>(&input)?);
            } else if lookahead.peek(kw::mutation) {
                args.mutation = Some(Self::parse_expression::<kw::mutation, Ident>(&input)?);
            } else if lookahead.peek(kw::query_path) {
                args.query_path = Some(Self::parse_expression::<kw::query_path, LitStr>(&input)?);
            } else if lookahead.peek(kw::schema_path) {
                args.schema_path = Some(Self::parse_expression::<kw::schema_path, LitStr>(&input)?);
            } else if lookahead.peek(kw::input_stream) {
                args.input_stream = Some(Self::parse_expression::<kw::input_stream, Expr>(&input)?);
            } else if lookahead.peek(kw::output_stream) {
                args.output_stream =
                    Some(Self::parse_expression::<kw::output_stream, Expr>(&input)?);
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
/// #[shopify_function(query_path = "input.graphql", schema_path = "schema.graphql")]
/// fn function(input: input::ResponseData) -> Result<output::FunctionResult> {
///     /* ... */
/// }
/// ```
///
/// Generation of GraphQL types
/// - `query_path`: A path to a GraphQL query, whose result will be used
///    as the input for the function invocation. The query MUST be named "Input".
/// - `schema_path`: A path to Shopify's GraphQL schema definition. You
///   can find it in the `example` folder of the repo, or use the CLI
///   to download a fresh copy.
///
/// Note: This macro creates a file called `.output.graphql` in the root
/// directory of the project. It can be safely added to your `.gitignore`. We
/// hope we can avoid creating this file at some point in the future.
///
/// By default, the function input is read from stdin and the result
/// is outputted to stdout. To override this, optional `input_stream`
/// and `output_stream` parameters can be set. These parameters must
/// implement the std::io::Read and std::io::Write traits respectively.
///
/// ```ignore
/// #[shopify_function(
///     query_path = "input.graphql",
///     schema_path = "schema.graphql",
///     input_stream = MyInputStream,
///     output_stream = MyOutputStream
/// )]
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

    let export: Ident = args.export.unwrap();
    let _export_string: String = export.to_string();
    let export_wrapper: Ident = Ident::new(&format!("export_{}", export), Span::mixed_site());
    let query: Ident = args.query.unwrap();
    let mutation: Ident = args.mutation.unwrap();
    let query_path: LitStr = args.query_path.unwrap();
    let schema_path: LitStr = args.schema_path.unwrap();

    let cargo_manifest_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap();

    // TODO: Separate from input type gen
    let mut output_query_path = Path::new(&cargo_manifest_dir).to_path_buf();
    let output_query_file_name: &str = &format!(".{}.output.graphql", export);
    output_query_path.push(output_query_file_name);

    std::fs::File::create(&output_query_path)
        .expect("Could not create output query file")
        .write_all(
            format!(
                "mutation {}($result: FunctionResult!) {{ handleResult(result: $result) }}",
                mutation
            )
            .as_bytes(),
        )
        .expect(&format!("Could not write to {}", output_query_file_name));

    let gen = quote! {
        #[derive(graphql_client::GraphQLQuery, Clone, Debug, serde::Deserialize, PartialEq)]
        #[serde(rename_all(deserialize = "camelCase"))]
        #[graphql(
            query_path = #query_path,
            schema_path = #schema_path,
            response_derives = "Clone,Debug,PartialEq,Eq,Deserialize",
            variables_derives = "Clone,Debug,PartialEq,Eq,Deserialize",
            skip_serializing_none
        )]
        struct #query;

        #[derive(graphql_client::GraphQLQuery, Clone, Debug, serde::Deserialize, PartialEq)]
        #[graphql(
            query_path = #output_query_file_name,
            schema_path = #schema_path,
            response_derives = "Clone,Debug,PartialEq,Eq,Deserialize",
            variables_derives = "Clone,Debug,PartialEq,Eq,Deserialize",
            skip_serializing_none
        )]
        struct #mutation;

        #[no_mangle]
        // #[export_name = #export_string]
        pub extern "C" fn #export_wrapper() -> ::shopify_function::Result<()> {
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

#[cfg(test)]
mod tests {}

mod kw {
    syn::custom_keyword!(export);
    syn::custom_keyword!(query);
    syn::custom_keyword!(mutation);
    syn::custom_keyword!(query_path);
    syn::custom_keyword!(schema_path);
    syn::custom_keyword!(input_stream);
    syn::custom_keyword!(output_stream);
}
