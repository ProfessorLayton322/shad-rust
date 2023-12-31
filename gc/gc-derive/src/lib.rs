use proc_macro::{TokenStream, TokenTree::Group, TokenTree::Ident};
#[proc_macro_derive(Scan)]
pub fn derive_scan(input: TokenStream) -> TokenStream {
    let mut iter = input.into_iter();
    iter.next();
    let Some(Ident(ident)) = iter.next() else {
        return TokenStream::new();
    };
    let default_answer: TokenStream = format!(
        "impl Scan for {} {{
        fn get_allocations(&self) -> Vec<usize> {{
            vec![]
        }}
    }}",
        ident
    )
    .parse()
    .unwrap();
    if let Some(Group(group)) = iter.next() {
        let Some(Ident(field_ident)) = group.stream().into_iter().next() else {
            return default_answer;
        };
        format!(
            "impl Scan for {} {{
            fn get_allocations(&self) -> Vec<usize> {{
                self.{}.get_allocations()
            }}
        }}",
            ident,
            field_ident
        )
        .parse()
        .unwrap()
    } else {
        default_answer
    }
}
