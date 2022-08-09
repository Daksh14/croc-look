use std::iter::Peekable;

use proc_macro2::{Delimiter, Ident, Span, TokenStream, TokenTree};

pub fn get_ident<T: AsRef<str>>(name: T) -> Ident {
    Ident::new(name.as_ref(), Span::call_site())
}

pub fn stream_to_string(vec: Vec<TokenTree>) -> String {
    let mut stream = TokenStream::new();
    stream.extend(vec.into_iter());
    stream.to_string()
}

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

pub fn loc_trait_impl(name: &str, code: TokenStream, impl_for: Option<&str>) -> Option<String> {
    fn find_after_for(
        iter: &mut Peekable<impl Iterator<Item = TokenTree>>,
        collection: &mut Vec<TokenTree>,
        impl_for: Option<&str>,
    ) -> Option<String> {
        while let Some(token) = iter.peek() {
            match token {
                TokenTree::Ident(ident) if ident == &get_ident("for") => {
                    if let Some(struct_ident) = impl_for {
                        iter.next();
                        if let Some(next_ident) = iter.next() {
                            match next_ident {
                                TokenTree::Ident(next_ident) => {
                                    if next_ident != struct_ident {
                                        return None;
                                    }
                                }
                                _ => return None,
                            }
                        }
                    }
                    while let Some(token) = iter.peek() {
                        match token {
                            TokenTree::Group(group) if group.delimiter() == Delimiter::Brace => {
                                let mut stream = TokenStream::new();
                                // UNWRAP: iter.peek() is some
                                collection.push(iter.next().unwrap());
                                stream.extend(get_after_impl(collection).into_iter());

                                return Some(stream.to_string());
                            }
                            // UNWRAP: iter.peek() is some
                            _ => collection.push(iter.next().unwrap()),
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
            TokenTree::Ident(ref ident) => {
                if ident == &get_ident("impl") {
                    let mut collection = vec![tree];
                    while let Some(x) = iter.peek() {
                        match x {
                            TokenTree::Ident(ident) if ident == &get_ident(name) => {
                                if let Some(x) =
                                    find_after_for(&mut iter, &mut collection, impl_for)
                                {
                                    return Some(x);
                                }
                            }
                            // UNWRAP: iter.peek() is some
                            _ => collection.push(iter.next().unwrap()),
                        }
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
