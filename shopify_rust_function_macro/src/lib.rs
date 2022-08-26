use proc_macro;
use proc_macro2::TokenStream;
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
        use serde::Serialize;
        use serde_json;

        fn main() -> Result<(), Box<dyn std::error::Error>> {
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

#[proc_macro_attribute]
pub fn input_query(
    attr: proc_macro::TokenStream,
    item: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    // let params = format!("{}", attr);
    let params = TokenStream::from(attr);
    let ast: syn::Item = syn::parse(item).unwrap();

    return quote! {
        #[derive(graphql_client::GraphQLQuery, Clone, Debug, serde::Deserialize)]
        #[serde(rename_all(deserialize = "camelCase"))]
        #[graphql(#params)]
        #ast
    }
    .into();
}
#[cfg(test)]
mod tests {}
