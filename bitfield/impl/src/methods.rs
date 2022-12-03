use proc_macro2::{TokenStream, Ident, Span};
use syn::TypePath;
use quote::quote;

pub fn getters(fields: &[(&Ident, &TypePath)]) -> TokenStream {
    let mut out = Vec::new();
    let fields_len = fields.len();

    for (field_index, (field_name, ty)) in fields.iter().enumerate() {
        let getter = Ident::new(&format!("get_{}", field_name), Span::call_site());
        let out_ty = quote! { <#ty as bitfield::Specifier>::InOutType };

        out.push(quote! {
            fn #getter(&self) -> #out_ty {
                let [start, len] = <Self as bitfield::Bitfield<#fields_len>>::BIT_INDICES[#field_index];
                let mut result: u64 = 0;

                for i in (start..(start + len)) {
                    let data_byte = self.data[i / 8];
                    let right_shift = 8 - (i % 8) - 1;

                    result = (result << 1) | (((data_byte >> right_shift) & 0b1) as u64);
                }

                unsafe { ::std::mem::transmute_copy(&result) }
            }
        });
    }

    TokenStream::from_iter(out)
}

pub fn setters(fields: &[(&Ident, &TypePath)]) -> TokenStream {
    let mut out = Vec::new();
    let fields_len = fields.len();

    for (field_index, (field_name, ty)) in fields.iter().enumerate() {
        let setter = Ident::new(&format!("set_{}", field_name), Span::call_site());
        let in_ty = quote! { <#ty as bitfield::Specifier>::InOutType };

        out.push(quote! {
            fn #setter(&mut self, value: #in_ty) -> &mut Self {
                let [start, len] = <Self as bitfield::Bitfield<#fields_len>>::BIT_INDICES[#field_index];

                let mut right_shift = 0;
                let val = value as u64;

                for i in (start..(start + len)).rev() {
                    let data_byte = self.data[i / 8];
                    let mut left_shift = 8 - (i % 8) - 1;
                    
                    self.data[i / 8] = data_byte | (
                        (((val >> right_shift) as u8) & 0b1) << left_shift
                    );

                    right_shift += 1;
                }

                self
            }
        });
    }

    TokenStream::from_iter(out)
}
