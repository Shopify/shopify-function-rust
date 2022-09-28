use std::io::Write;
use std::path::Path;

use proc_macro2::TokenStream;
use proc_macro2::TokenTree;
use quote::quote;
use syn::{self, FnArg};

#[proc_macro_attribute]
pub fn shopify_function(
    _attr: proc_macro::TokenStream,
    item: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    let ast: syn::ItemFn = syn::parse(item).unwrap();

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

    let gen = quote! {
        fn main() -> Result<()> {
            let input: #input_type = serde_json::from_reader(std::io::BufReader::new(std::io::stdin()))?;
            let mut out = std::io::stdout();
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
        .expect(format!("No attribute with name {} found", attr).as_str());
    let value = attrs
        .get(attr_index + 2)
        .expect(format!("No value given for {} attribute", attr).as_str())
        .to_string();
    value.as_str()[1..value.len() - 1].to_string()
}

#[proc_macro_attribute]
pub fn input_query(
    attr: proc_macro::TokenStream,
    item: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    let params = TokenStream::from(attr);
    let ast: syn::Item = syn::parse(item).unwrap();

    let cargo_manifest_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap();
    let schema_path = extract_attr(&params, "schema_path");

    let mut output_query_path = Path::new(&cargo_manifest_dir).to_path_buf();
    output_query_path.push(".output.query");
    std::fs::File::create(&output_query_path)
        .expect("Could not create .output.query")
        .write_all(
            r"
                mutation Output(
                    $result: FunctionResult!
                ) {
                    handleResult(result: $result)
                }
            "
            .as_bytes(),
        )
        .expect("Could not write to .output.query");

    return quote! {
        #[derive(graphql_client::GraphQLQuery, Clone, Debug, serde::Deserialize, PartialEq)]
        #[serde(rename_all(deserialize = "camelCase"))]
        #[graphql(
            #params,
            response_derives = "Clone,Debug,PartialEq,Deserialize",
            variables_derives = "Clone,Debug,PartialEq,Deserialize",
            skip_serializing_none
        )]
        struct InputQuery;

        #[derive(graphql_client::GraphQLQuery, Clone, Debug, serde::Deserialize, PartialEq)]
        #[graphql(
            query_path = "./.output.query",
            schema_path = #schema_path,
            response_derives = "Clone,Debug,PartialEq,Deserialize",
            variables_derives = "Clone,Debug,PartialEq,Deserialize",
            skip_serializing_none
        )]
        struct Output;

        #ast
    }
    .into();
}

#[cfg(test)]
mod tests {}
