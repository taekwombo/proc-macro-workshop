use syn::{Data, DeriveInput, Expr, Field, Fields, Type, TypePath, parse::Error};
use proc_macro2::{Ident, TokenStream};

static INVALID_BITFIELD_ATTR_INPUT: &str = "Expected:
    #[bitfield]
    struct Name {{
        field_name: <bitfield::Specifier>,
    }}
";

static INVALID_BITFIELD_SPEC_INPUT: &str = "Expected:
    #[derive(BitfieldSpecifier)]
    enum Name {{
        VariantName,
        VariantName = discriminator,
    }}
";

fn get_field_name(field: &Field) -> Result<&Ident, TokenStream> {
    field.ident.as_ref().ok_or_else(|| 
            Error::new_spanned(field, INVALID_BITFIELD_ATTR_INPUT).to_compile_error())
}

pub fn get_field_type(field: &Field) -> Result<&TypePath, TokenStream> {
    match field.ty {
        Type::Path(ref t) => Ok(t),
        ref ty => Err(Error::new_spanned(ty, INVALID_BITFIELD_ATTR_INPUT).to_compile_error()),
    }
}

pub fn get_struct_fields(input: & DeriveInput) -> Result<(Vec<&Field>, Vec<(&Ident, &TypePath)>), TokenStream> {
    let struct_fields = match input.data {
        Data::Struct(ref d) => &d.fields,
        _ => return Err(Error::new_spanned(input, INVALID_BITFIELD_ATTR_INPUT).to_compile_error()),
    };

    let fields = match struct_fields {
        Fields::Named(n) => n,
        _ => return Err(Error::new_spanned(struct_fields, INVALID_BITFIELD_ATTR_INPUT).to_compile_error()),
    };

    let fields = fields.named.iter().collect::<Vec<_>>();

    let mut output = Vec::new();

    for field in fields.iter() {
        let field_name = get_field_name(field)?;
        let field_type = get_field_type(field)?;

        output.push((field_name, field_type));
    }

    if output.is_empty() {
        return Err(Error::new_spanned(input, INVALID_BITFIELD_ATTR_INPUT).to_compile_error());
    }

    Ok((fields, output))
}

pub fn get_enum_variants(input: &DeriveInput) -> Result<Vec<(&Ident, Option<&Expr>)>, TokenStream> {
    let mut variants = Vec::new();

    let data_enum = match input.data {
        Data::Enum(ref e) => e,
        _ => return Err(Error::new_spanned(input, INVALID_BITFIELD_SPEC_INPUT).to_compile_error()),
    };

    if data_enum.variants.is_empty() {
        return Err(Error::new_spanned(input, INVALID_BITFIELD_SPEC_INPUT).to_compile_error());
    }

    for v in data_enum.variants.iter() {
        variants.push((
            &v.ident,
            v.discriminant.as_ref().map(|(_, e)| e),
        ));
    }

    Ok(variants)
}
