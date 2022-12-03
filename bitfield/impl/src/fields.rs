use syn::{Data, DeriveInput, Field, Fields, Type, TypePath, parse::Error};
use proc_macro2::{Ident, TokenStream};

static INVALID_INPUT_TYPE: &str = "Expected:
    struct Name {{
        field_name: <bitfield::Specifier>,
    }}.
";

fn get_field_name(field: &Field) -> Result<&Ident, TokenStream> {
    field.ident.as_ref().ok_or_else(|| 
            Error::new_spanned(field, INVALID_INPUT_TYPE).to_compile_error())
}

fn get_field_type(field: &Field) -> Result<&TypePath, TokenStream> {
    match field.ty {
        Type::Path(ref t) => Ok(t),
        ref ty => Err(Error::new_spanned(ty, INVALID_INPUT_TYPE).to_compile_error()),
    }
}

pub fn get_struct_fields(input: & DeriveInput) -> Result<Vec<(&Ident, &TypePath)>, TokenStream> {
    let struct_fields = match input.data {
        Data::Struct(ref d) => &d.fields,
        _ => return Err(Error::new_spanned(input, INVALID_INPUT_TYPE).to_compile_error()),
    };

    let fields = match struct_fields {
        Fields::Named(n) => n,
        _ => return Err(Error::new_spanned(struct_fields, INVALID_INPUT_TYPE).to_compile_error()),
    };

    let mut output = Vec::new();

    for field in fields.named.iter() {
        let field_name = get_field_name(field)?;
        let field_type = get_field_type(field)?;

        output.push((field_name, field_type));
    }

    if output.is_empty() {
        return Err(Error::new_spanned(input, INVALID_INPUT_TYPE).to_compile_error());
    }

    Ok(output)
}

