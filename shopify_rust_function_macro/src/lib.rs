use proc_macro::TokenStream;
use quote::quote;
use syn;

#[proc_macro_attribute]
pub fn shopify_function(attr: TokenStream, item: TokenStream) -> TokenStream {
    let ast: syn::ItemFn = syn::parse(item).unwrap();

    let name = &ast.sig.ident;

    let gen = quote! {
        fn main() -> Result<(), Box<dyn std::error::Error>> {
            let input: input::Input = serde_json::from_reader(std::io::BufReader::new(std::io::stdin()))?;
            let mut out = std::io::stdout();
            let mut serializer = serde_json::Serializer::new(&mut out);
            #name(input)?.serialize(&mut serializer)?;
            Ok(())
        }
        #ast
    };

    gen.into()
}

#[cfg(test)]
mod tests {}
