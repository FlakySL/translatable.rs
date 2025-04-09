use std::collections::HashMap;

use proc_macro2::TokenStream as TokenStream2;
use quote::{ToTokens, quote};

pub fn map_to_tokens<K: ToTokens, V: ToTokens>(map: &HashMap<K, V>) -> TokenStream2 {
    let map = map.into_iter().map(|(key, value)| {
        let key = key.into_token_stream();
        let value = value.into_token_stream();

        quote! { (#key, #value) }
    });

    quote! {
        vec![#(#map),*]
            .iter()
            .collect::<HashMap<_, _>>()
    }
}
