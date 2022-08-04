use loading::Loading;
use proc_macro2::{Ident, Span, TokenStream, TokenTree};

macro_rules! ident {
    ( $string : expr ) => {
        Ident::new($string, Span::call_site())
    };
}

pub fn get_derive(name: &str, code: String, loading: &Loading) -> Option<String> {
    // UNWRAP: code from cargo expand is gonna be valid rust code
    let parsed: TokenStream = syn::parse_str(&code).unwrap();

    loc_derive(name, parsed, loading)
}

pub fn get_struct(name: &str, code: String, loading: &Loading) -> Option<String> {
    // UNWRAP: code from cargo expand is gonna be valid rust code
    let parsed: TokenStream = syn::parse_str(&code).unwrap();

    loc_struct(name, parsed, loading)
}

fn loc_derive(name: &str, code: TokenStream, loading: &Loading) -> Option<String> {
    let mut iter = code.into_iter().peekable();

    while let Some(tree) = iter.next() {
        match tree {
            TokenTree::Group(group) => {
                if let Some(string) = loc_derive(name, group.stream(), loading) {
                    return Some(string);
                }
            }
            TokenTree::Ident(ref ident) => {
                if ident == &ident!(name) {
                    if let Some(TokenTree::Ident(ident)) = iter.peek() {
                        if ident == &ident!("for") {
                            iter.next();
                            let what_for = iter.next();
                            if let Some(TokenTree::Group(group)) = iter.peek() {
                                // UNWRAP: If iter.next().next() is Some then what_for is Some
                                loading.success(format!("Loading impl for {}", what_for.unwrap()));
                                return Some(group.stream().to_string());
                            }
                        }
                    }
                }
            }
            _ => (),
        }
    }
    None
}

fn loc_struct(name: &str, code: TokenStream, loading: &Loading) -> Option<String> {
    let mut iter = code.into_iter().peekable();

    while let Some(tree) = iter.next() {
        match tree {
            TokenTree::Group(group) => {
                if let Some(string) = loc_derive(name, group.stream(), loading) {
                    return Some(string);
                }
            }
            TokenTree::Ident(ref ident) => {
                if ident == &ident!("struct") {
                    if let Some(TokenTree::Ident(ident)) = iter.peek() {
                        if ident == &ident!(name) {
                            let clone = ident.clone();
                            iter.next();
                            let mut collection = vec![tree, TokenTree::Ident(clone)];

                            for x in iter.by_ref() {
                                if let TokenTree::Group(_) = x {
                                    collection.push(x);

                                    loading.success(format!("Loading struct {}", name));

                                    let mut stream = TokenStream::new();
                                    stream.extend(collection.into_iter());

                                    return Some(stream.to_string());
                                } else {
                                    collection.push(x);
                                }
                            }
                        }
                    }
                }
            }
            _ => (),
        }
    }

    None
}
