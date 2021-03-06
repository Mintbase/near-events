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
        #[cfg_attr(feature = "ser", derive(near_sdk::serde::Serialize))]
        #[cfg_attr(feature = "de", derive(near_sdk::serde::Deserialize))]
        #[cfg_attr(
            any(feature = "ser", feature = "de"),
            serde(crate = "near_sdk::serde")
        )]
    };

    // implement direct log -> event serialization
    let event_impl = quote::quote_spanned! {Span::call_site()=>
        #[cfg(feature = "ser")]
        impl #name {
            pub fn serialize_event(self) -> String {
                let data = near_sdk::serde_json::value::to_value(self).unwrap();
                near_events::serialize_from_value(#standard, #version, #event, data)
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
