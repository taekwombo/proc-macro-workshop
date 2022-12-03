use syn::{DeriveInput, Data, Field, Fields, Type, FieldsNamed, PathArguments, parse::Error};
use quote::ToTokens;
use proc_macro2::{TokenStream, Ident};
use std::collections::HashMap;

static INVALID_FIELD_TYPE: &str = "Expected one of bitfield B1, B2, ... types.";
static INVALID_INPUT_TYPE: &str = "Expected:
    struct Name {{
        field_name: <bitfield::Specifier>,
    }}.
";

fn invalid_input<T: ToTokens, U: std::fmt::Display>(tokens: &T, msg: U) -> TokenStream {
    Error::new_spanned(tokens, msg).to_compile_error()
}

fn get_named_fields<'a>(fields: &'a Fields) -> Result<&'a FieldsNamed, TokenStream> {
    match fields {
        Fields::Named(n) => Ok(n),
        _ => return Err(invalid_input(fields, INVALID_INPUT_TYPE)),
    }
}

fn get_field_name(field: &Field) -> Result<&Ident, TokenStream> {
    field.ident.as_ref().ok_or_else(|| Error::new_spanned(field, invalid_input(field, INVALID_INPUT_TYPE)).to_compile_error())
}

fn get_field_type<'a>(field: &'a Field, map: &HashMap<Ident, u8>) -> Result<&'a Ident, TokenStream> {
    let type_path = match field.ty {
        Type::Path(ref t) => t,
        ref ty @ _ => return Err(invalid_input(ty, INVALID_FIELD_TYPE)),
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

pub fn get_struct_fields<'a>(input: &'a DeriveInput, map: &HashMap<Ident, u8>) -> Result<Vec<(&'a Ident, &'a Ident)>, TokenStream> {
    let struct_fields = match input.data {
        Data::Struct(ref d) => &d.fields,
        _ => return Err(invalid_input(input, INVALID_INPUT_TYPE)),
    };

    let fields = get_named_fields(struct_fields)?;

    let mut output: Vec<(&Ident, &Ident)> = Vec::new();

    for field in fields.named.iter() {
        output.push((get_field_name(field)?, get_field_type(field, &map)?));
    }

    Ok(output)
}

pub fn calculate_size(fields: &Vec<(&Ident, &Ident)>, map: &HashMap<Ident, u8>) -> Result<usize, TokenStream> {
    let mut size = 0usize;

    for (ident, ty) in fields {
        let entry = map.get(ty);
    
        if entry.is_none() {
            return Err(invalid_input(ident, INVALID_INPUT_TYPE));
        }

        size += usize::from(*entry.unwrap());

    }

    Ok(size)
}

