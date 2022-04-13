use itertools::Itertools;
use proc_macro::{Literal, TokenStream, TokenTree};
use proc_macro2::{Ident, Span};
use std::collections::HashMap;

#[proc_macro_attribute]
pub fn near_event_data_log(
    macro_args: TokenStream,
    input: TokenStream,
) -> TokenStream {
    let (standard, version, event) = parse_macro_args(macro_args);

    let (name, attrs, base_impl) = parse_typedef(input);

    // add additional attributes
    let serde_attrs = quote::quote_spanned! {Span::call_site()=>
        #[derive(Serialize)]
        #[serde(crate = "near_sdk::serde")]
    };

    //
    let event_impl = quote::quote_spanned! {Span::call_site()=>
        impl NearEventDataLog for #name {
            fn serialize_event(&self) -> String {
                serialize(#standard, #version, #event, vec![self])
            }
        }
    };

    // TODO: support for
    //  - indexing (Deserialize) and
    //  - legacy indexing (no serialize)
    // where both need to be feature gated such that they do not end up being
    // wasm'ed
    quote::quote_spanned! {Span::call_site()=>
        #attrs
        #serde_attrs
        #base_impl

        #event_impl
    }
    .into()
}

fn parse_macro_args(args: TokenStream) -> (String, String, String) {
    let assignments = parse_assignment_list(args);
    assert!(assignments.len() == 3);

    let standard = match assignments.get("standard") {
        Some(literal) => literal.to_string(),
        _ => panic!("You need to specify a standard for the event"),
    };

    let version = match assignments.get("version") {
        Some(literal) => literal.to_string(),
        _ => panic!("You need to specify a version for the event"),
    };

    let event = match assignments.get("event") {
        Some(literal) => literal.to_string(),
        _ => panic!("You need to specify an event name for the event"),
    };

    (
        unstringify_token(standard),
        unstringify_token(version),
        unstringify_token(event),
    )
}

// ------------------ helpers for parsing type definitions ------------------ //
fn parse_typedef(
    input: TokenStream,
) -> (Ident, proc_macro2::TokenStream, proc_macro2::TokenStream) {
    let mut input_iter = input.into_iter();
    let mut attrs = vec![];
    let mut typedef = vec![];
    let mut started_def = false;

    // get attributes
    while !started_def {
        match input_iter.next() {
            Some(TokenTree::Ident(ident))
                if ["struct", "enum"].contains(&ident.to_string().as_str()) =>
            {
                typedef.push(TokenTree::Ident(ident));
                started_def = true;
            }
            Some(token) => attrs.push(token),
            None => panic!("Cannot parse unfinished typedef",),
        }
    }

    // get name
    let name = match input_iter.next() {
        Some(TokenTree::Ident(name)) => {
            typedef.push(TokenTree::Ident(name.clone()));
            Ident::new(&name.to_string(), name.span().into())
        }
        _ => panic!("Cannot parse typedef without name"),
    };

    // get body
    typedef.extend(input_iter);

    (
        name,
        TokenStream::from_iter(attrs.into_iter()).into(),
        TokenStream::from_iter(typedef.into_iter()).into(),
    )
}

// --------------------- helpers for parsing attr args ---------------------- //
fn parse_assignment_list(list: TokenStream) -> HashMap<String, Literal> {
    // let mut v = Vec::with_capacity(3);
    let mut map = HashMap::with_capacity(3);
    // split into triplets
    for mut assignment in split_by_commas(list) {
        assert!(assignment.len() == 3);

        // get the value
        let value = match assignment.pop().unwrap() {
            TokenTree::Literal(value) => value,
            tt => panic!("Need to assign a literal, got: {}", tt),
        };
        // ditch the equals token
        let _ = match assignment.pop().unwrap() {
            TokenTree::Punct(p) if p.as_char() == '=' => {}
            tt => panic!("Assignment needs an '=', got: {}", tt),
        };
        // get the identifier
        let ident = match assignment.pop().unwrap() {
            TokenTree::Ident(ident) => ident.to_string(),
            tt => panic!("Need to assign to an identifier, got: {}", tt),
        };

        map.insert(ident, value);
    }
    map
}

fn split_by_commas(list: TokenStream) -> Vec<Vec<TokenTree>> {
    let mut helper_index = 0;
    list.into_iter()
        .group_by(|token| {
            // this simply skips the commas
            if helper_index % 2 != 0 {
                helper_index += 1;
            }
            if let TokenTree::Punct(p) = token {
                if p.as_char() == ',' {
                    helper_index += 1;
                }
            }
            helper_index
        })
        .into_iter()
        .step_by(2)
        .map(|(_, group)| group.into_iter().collect())
        .collect()
}

fn unstringify_token(s: String) -> String {
    s.replace('"', "")
}
