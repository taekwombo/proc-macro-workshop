use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput};

mod util;

use util::*;

#[proc_macro_derive(CustomDebug, attributes(debug))]
pub fn derive(input: TokenStream) -> TokenStream {
    let derive_input = parse_macro_input!(input as DeriveInput);
    let data_struct = match get_data_struct(&derive_input) {
        Ok(v) => v,
        Err(e) => return e.into(),
    };
    let fields = match get_fields_debug(data_struct) {
        Ok(f) => f,
        Err(e) => return e.into(),
    };

    let DeriveInput {
        ident,
        generics,
        attrs,
        ..
    } = &derive_input;
    let fmt_name = ident.to_string();
    let where_clause =
        where_clause_from_attrs(attrs).unwrap_or_else(|| generate_where_clause(&fields, generics));
    let debug_field_calls = proc_macro2::TokenStream::from_iter(fields.iter().map(
        |FieldDebug {
             ident, name, debug, ..
         }| {
            let value = debug
                .as_ref()
                .map(|d| quote! { &::std::format_args!(#d, &self.#ident) })
                .unwrap_or_else(|| quote! { &self.#ident });

            quote! {
                .field(#name, #value)
            }
        },
    ));

    let generics_unbounded = strip_bounds_from_generics(generics);

    quote! {
        impl #generics ::std::fmt::Debug for #ident #generics_unbounded #where_clause {
            fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::result::Result<(), ::std::fmt::Error> {
               f
                   .debug_struct(#fmt_name)
                   #debug_field_calls
                   .finish()
            }
        }
    }.into()
}
