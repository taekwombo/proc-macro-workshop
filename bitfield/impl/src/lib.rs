use proc_macro::TokenStream;
use proc_macro2::{Ident, Span};
use std::collections::HashMap;

mod input;

fn build_ident_map() -> HashMap<Ident, u8> {
    (1_u8..=64)
        .map(|v| (v, Ident::new(&format!("B{}", v), Span::call_site())))
        .fold(HashMap::new(), |mut map, (bits, ident)| {
            map.insert(ident, bits);
            return map;
        })
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
    let fields = get_ok!(input::get_struct_fields(&input, &ident_map));
    let bit_size = get_ok!(input::calculate_size(&fields, &ident_map));
    let byte_size = bit_size / 8;

    let DeriveInput { ident, vis, .. } = input;

    quote! {
        #[repr(C)]
        #vis struct #ident {
            data: [u8; #byte_size],
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
