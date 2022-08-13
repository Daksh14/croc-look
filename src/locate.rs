use std::iter::Peekable;

use proc_macro2::{Delimiter, Ident, Span, TokenStream, TokenTree};

/// helper function to generate ident from string
pub fn get_ident<T: AsRef<str>>(name: T) -> Ident {
    Ident::new(name.as_ref(), Span::call_site())
}

/// helper function to map a stream to string
pub fn stream_to_string(vec: Vec<TokenTree>) -> String {
    let mut stream = TokenStream::new();
    stream.extend(vec.into_iter());
    stream.to_string()
}

/// the locate functions store a "buffer" that contains some tokens.
/// This filters out those tokens, looks for the ident "impl" and gets
/// everything after that in a tokenstream
pub fn get_after_impl(vec: &[TokenTree]) -> Vec<TokenTree> {
    let mut index = 0;
    vec.iter().enumerate().for_each(|(inner_index, value)| {
        if let TokenTree::Ident(ident) = value {
            if ident == &get_ident("impl") {
                index = inner_index;
            }
        }
    });

    let slice = &vec[index..];
    slice.to_vec()
}

/// Locate the trait which we are looking for, match with struct if given
pub fn loc_trait_impl(name: &str, code: TokenStream, impl_for: Option<&str>) -> Option<String> {
    fn find_after_for(
        iter: &mut Peekable<impl Iterator<Item = TokenTree>>,
        collection: &mut Vec<TokenTree>,
        impl_for: Option<&str>,
    ) -> Option<String> {
        while let Some(token) = iter.peek() {
            match token {
                // find the ident "for"
                TokenTree::Ident(ident) if ident == &get_ident("for") => {
                    // check if struct is provided
                    if let Some(struct_ident) = impl_for {
                        collection.push(iter.next().unwrap());

                        match iter.peek() {
                            Some(TokenTree::Ident(next_ident)) => {
                                if next_ident != struct_ident {
                                    // if it's not the struct we're looking for, return early
                                    return None;
                                }
                            }
                            // since we have to look for a particular struct here, we get to be
                            // strict
                            _ => return None,
                        }
                    }
                    // grab the rest
                    while let Some(token) = iter.next() {
                        collection.push(iter.next().unwrap());
                        match token {
                            TokenTree::Group(group) if group.delimiter() == Delimiter::Brace => {
                                let mut stream = TokenStream::new();
                                // UNWRAP: iter.peek() is some
                                collection.push(iter.next().unwrap());
                                stream.extend(get_after_impl(collection).into_iter());

                                return Some(stream.to_string());
                            }
                            _ => (),
                        }
                    }
                }
                // UNWRAP: iter.peek() is some
                _ => collection.push(iter.next().unwrap()),
            }
        }
        None
    }

    let mut iter = code.into_iter().peekable();

    while let Some(tree) = iter.next() {
        match tree {
            TokenTree::Group(group) => {
                if let Some(string) = loc_trait_impl(name, group.stream(), impl_for) {
                    return Some(string);
                }
            }
            TokenTree::Ident(ref ident) if ident == &get_ident("impl") => {
                let mut collection = vec![tree];

                while let Some(x) = iter.next() {
                    match x {
                        TokenTree::Ident(ref ident) if ident == &get_ident(name) => {
                            collection.push(x);
                            if let Some(x) = find_after_for(&mut iter, &mut collection, impl_for) {
                                return Some(x);
                            }
                        }
                        TokenTree::Group(group) => {
                            if let Some(string) = loc_trait_impl(name, group.stream(), impl_for) {
                                return Some(string);
                            }
                        }
                        _ => collection.push(x),
                    }
                }
            }
            _ => (),
        }
    }

    None
}

pub fn loc_struct(name: &str, code: TokenStream) -> Option<String> {
    let mut iter = code.into_iter().peekable();

    while let Some(tree) = iter.next() {
        match tree {
            TokenTree::Group(group) => {
                if let Some(string) = loc_struct(name, group.stream()) {
                    return Some(string);
                }
            }
            TokenTree::Ident(ref ident) => {
                if ident == &get_ident("struct") {
                    if let Some(TokenTree::Ident(ident)) = iter.peek() {
                        if ident == &get_ident(name) {
                            let clone = ident.clone();
                            iter.next();
                            let mut collection = vec![tree, TokenTree::Ident(clone)];

                            for x in iter.by_ref() {
                                if let TokenTree::Group(_) = x {
                                    collection.push(x);
                                    if let Some(TokenTree::Punct(punct)) = iter.peek() {
                                        if ';' == punct.as_char() {
                                            collection.push(iter.next().unwrap());
                                        }
                                    }

                                    return Some(stream_to_string(collection));
                                }
                                collection.push(x);
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

pub fn loc_function(name: &str, code: TokenStream) -> Option<String> {
    let mut iter = code.into_iter().peekable();

    while let Some(tree) = iter.next() {
        match tree {
            TokenTree::Group(group) => {
                if let Some(string) = loc_function(name, group.stream()) {
                    return Some(string);
                }
            }
            TokenTree::Ident(ref ident) => {
                if ident == &get_ident("fn") {
                    if let Some(TokenTree::Ident(ident)) = iter.peek() {
                        if ident == &get_ident(name) {
                            let clone = ident.clone();
                            iter.next();
                            let mut collection = vec![tree, TokenTree::Ident(clone)];

                            for x in iter.by_ref() {
                                if let TokenTree::Group(ref grp) = x {
                                    let delimiter = grp.delimiter();

                                    collection.push(x);

                                    // check if we arrived at the function body
                                    if delimiter == Delimiter::Brace {
                                        return Some(stream_to_string(collection));
                                    }
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
