#![feature(once_cell)]

use proc_macro::TokenStream;
use proc_macro2::{Ident, Span};
use std::collections::HashMap;
use std::sync::Once;

mod bits;

static mut IDENT_MAP: Option<HashMap<Ident, u8>> = None;
static INIT: Once = Once::new();

fn build_ident_map() -> &'static HashMap<Ident, u8> {
    unsafe {
        INIT.call_once(|| {
            let map = (1_u8..=64)
                .map(|v| (v, Ident::new(&format!("B{}", v), Span::call_site())))
                .fold(HashMap::new(), |mut map, (bits, ident)| {
                    map.insert(ident, bits);
                    map
                });
            IDENT_MAP.replace(map);
        });

        IDENT_MAP.as_ref().unwrap()
    }
}

macro_rules! get_ok {
    ($e:expr) => {
        match $e {
            Ok(v) => v,
            Err(e) => return e.into(),
        }
    }
}

#[proc_macro_attribute]
pub fn bitfield(args: TokenStream, input: TokenStream) -> TokenStream {
    use syn::{parse_macro_input, AttributeArgs, DeriveInput};
    use quote::quote;

    let _args = parse_macro_input!(args as AttributeArgs);
    let input = parse_macro_input!(input as DeriveInput);

    let ident_map = build_ident_map();
    let fields = get_ok!(bits::get_struct_fields(&input, ident_map));
    let (_, bytes) = get_ok!(bits::calculate_total_size(&fields, ident_map));

    let methods = bits::generate_methods(&fields, ident_map);

    let DeriveInput { ident, vis, .. } = input;

    quote! {
        #[repr(C)]
        #[derive(Debug, Clone)]
        #vis struct #ident {
            data: [u8; #bytes],
        }

        impl #ident {
            fn new() -> Self {
                Self {
                    data: [0; #bytes],
                }
            }

            #methods
        }
    }.into()
}

#[proc_macro]
pub fn bitfield_types(_input: TokenStream) -> TokenStream {
    use quote::quote;

    let types = build_ident_map()
        .iter()
        .map(|(ident, bits)| quote! {
            pub struct #ident;

            impl Specifier for #ident {
                const BITS: u8 = #bits;
            }
        })
        .collect::<Vec<_>>();

    quote! {
        pub trait Specifier {
            const BITS: u8;
        }

        #(#types)*
    }
    .into()
}
