use proc_macro::TokenStream;
use proc_macro2::Span;
use proc_macro_support::*;

#[proc_macro_attribute]
pub fn near_event_data(
    macro_args: TokenStream,
    input: TokenStream,
) -> TokenStream {
    let (standard, version, event) = parse_event_macro_args(macro_args.into());
    let (name, attrs, typedef) = parse_typedef(input.into());

    // add additional attributes
    let serde_attrs = quote::quote_spanned! {Span::call_site()=>
        #[derive(Serialize)]
        #[serde(crate = "near_sdk::serde")]
    };

    // implement direct log -> event serialization
    let event_impl = quote::quote_spanned! {Span::call_site()=>
        impl NearEventData for #name {
            fn serialize_event(self) -> String {
                // This implicitly assumes a tuple struct with its first member
                // being a vector
                serialize(#standard, #version, #event, self.0)
            }
        }
    };

    quote::quote_spanned! {Span::call_site()=>
        #attrs
        #serde_attrs
        #typedef

        #event_impl
    }
    .into()
}