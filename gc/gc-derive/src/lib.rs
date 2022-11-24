use proc_macro::{TokenStream};
use quote::{quote};

#[proc_macro_derive(Scan)]
pub fn derive_scan(input: TokenStream) -> TokenStream {
    let tokens = proc_macro2::TokenStream::from(input);
    let mut iter = tokens.into_iter();
    iter.next();
    let Some(proc_macro2::TokenTree::Ident(ident)) = iter.next() else {
        return TokenStream::new();
    };
    if let Some(proc_macro2::TokenTree::Group(group)) = iter.next() {
        let field = group.stream().into_iter().next().unwrap();
        let construct = quote! {
            impl Scan for #ident {
                fn get_allocations(&self) -> Vec<usize> {
                    self.#field.get_allocations()
                }
            }
        };
        TokenStream::from(construct)
    } else {
        let construct = quote! {
            impl Scan for #ident {
                fn get_allocations(&self) -> Vec<usize> {
                    vec![]
                }
            }
        };
        TokenStream::from(construct)
    }
}
