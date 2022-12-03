use syn::{DeriveInput, Data, Field, Fields, Type, FieldsNamed, PathArguments, parse::Error};
use quote::ToTokens;
use proc_macro2::{TokenStream, Ident};
use std::collections::HashMap;
use std::fmt::Display;

mod methods;

static INVALID_FIELD_TYPE: &str = "Expected one of bitfield B1, B2, ... types.";
static INVALID_INPUT_TYPE: &str = "Expected:
    struct Name {{
        field_name: <bitfield::Specifier>,
    }}.
";

fn invalid_input<T: ToTokens, U: Display>(tokens: &T, msg: U) -> TokenStream {
    Error::new_spanned(tokens, msg).to_compile_error()
}

fn get_named_fields(fields: &Fields) -> Result<&FieldsNamed, TokenStream> {
    match fields {
        Fields::Named(n) => Ok(n),
        _ => Err(invalid_input(fields, INVALID_INPUT_TYPE)),
    }
}

fn get_field_name(field: &Field) -> Result<&Ident, TokenStream> {
    field.ident.as_ref().ok_or_else(|| Error::new_spanned(field, invalid_input(field, INVALID_INPUT_TYPE)).to_compile_error())
}

fn get_field_type<'a>(
    field: &'a Field,
    map: &HashMap<Ident, u8>
) -> Result<&'a Ident, TokenStream> {
    let type_path = match field.ty {
        Type::Path(ref t) => t,
        ref ty => return Err(invalid_input(ty, INVALID_FIELD_TYPE)),
    };

    if type_path.qself.is_some() {
        return Err(invalid_input(type_path, INVALID_FIELD_TYPE));
    }

    let path = &type_path.path;

    if path.leading_colon.is_some() {
        return Err(invalid_input(path, INVALID_FIELD_TYPE));
    }

    if path.segments.len() != 1 {
        return Err(invalid_input(path, INVALID_FIELD_TYPE));
    }

    let path_seg = path.segments.iter().next().unwrap();

    let type_ident = match path_seg.arguments {
        PathArguments::None => &path_seg.ident,
        _ => return Err(invalid_input(path, INVALID_FIELD_TYPE)),
    };

    if !map.contains_key(type_ident) {
        return Err(invalid_input(type_ident, INVALID_FIELD_TYPE));
    }
    
    Ok(type_ident)
}

pub fn get_struct_fields<'a>(
    input: &'a DeriveInput,
    map: &HashMap<Ident, u8>
) -> Result<Vec<(&'a Ident, &'a Ident)>, TokenStream> {
    let struct_fields = match input.data {
        Data::Struct(ref d) => &d.fields,
        _ => return Err(invalid_input(input, INVALID_INPUT_TYPE)),
    };

    let fields = get_named_fields(struct_fields)?;

    let mut output: Vec<(&Ident, &Ident)> = Vec::new();

    for field in fields.named.iter() {
        output.push((get_field_name(field)?, get_field_type(field, map)?));
    }

    Ok(output)
}

pub fn calculate_total_size(fields: &[(&Ident, &Ident)], map: &HashMap<Ident, u8>) -> Result<(usize, usize), TokenStream> {
    let mut bits = 0usize;

    for (ident, ty) in fields {
        let entry = map.get(ty);
    
        if entry.is_none() {
            return Err(invalid_input(ident, INVALID_INPUT_TYPE));
        }

        bits += usize::from(*entry.unwrap());

    }

    let mut bytes = bits / 8;

    if bits % 8 != 0 {
        bytes += 1;
    }

    Ok((bits, bytes))
}

pub fn generate_methods(fields: &[(&Ident, &Ident)], map: &HashMap<Ident, u8>) -> TokenStream {
    use quote::quote;

    let mut methods: Vec<TokenStream> = Vec::new();

    // Track sum of bits for processed fields.
    let mut bit_sum = 0usize;

    for (field_name, field_type) in fields {
        let bit_size = *map.get(field_type).unwrap();
        let (value, ty) = methods::matching_value_type(bit_size);
        let (getter, setter) = methods::names(field_name);

        let bytes = methods::field_bytes(bit_sum, bit_size.into());

        let mut getter_values = Vec::new();
        let mut setter_values = Vec::new();
        let mut field_bits = 0_usize;

        for b in bytes.iter().rev() {
            getter_values.push(methods::get_data_byte(field_bits, &ty, b));
            setter_values.push(methods::set_data_byte(field_bits, b));

            field_bits += b.len;
        }

        methods.push(quote! {
            fn #getter(&self) -> #ty {
                #value #(| #getter_values)*
            }
            fn #setter(&mut self, value: #ty) -> &mut Self {
                #(#setter_values)*
                self
            }
        });

        bit_sum += field_bits;
    }

    quote!{ #(#methods)* }
}
