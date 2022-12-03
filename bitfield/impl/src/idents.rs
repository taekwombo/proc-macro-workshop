use proc_macro2::{Ident, Span};

fn to_pascal(name: &Ident) -> String {
    let mut s = String::new();

    let input = name.to_string();
    let mut chars = input.chars();

    s.push_str(&chars.next().unwrap().to_uppercase().to_string());

    for c in chars {
        if c.is_uppercase() {
            s.push('_');
            s.push(c);
        } else {
            s.push(c.to_ascii_uppercase());
        }
    }

    s
}

pub fn size_const(name: &Ident) -> Ident {
    Ident::new(
        &format!("__BITFIELD_SIZE__{}", to_pascal(name)),
        Span::call_site(),
    )
}

pub fn size_const_mod(name: &Ident) -> Ident {
    Ident::new(
        &format!("__BITFIELD_SIZE_MOD__{}", to_pascal(name)),
        Span::call_site(),
    )
}

/// Contains specifier functions for BitfieldSpecifier related items.
pub mod specifier {
    use super::*;

    pub fn max_value(name: &Ident) -> Ident {
        Ident::new(
            &format!("__BITFIELD_SPECIFIER_MAX_VALUE__{}", to_pascal(name)),
            Span::call_site(),
        )
    }

    pub fn discriminant_value(enum_name: &Ident, variant_name: &Ident) -> Ident {
        Ident::new(
            &format!("__BITFIELD_SPECIFIER_DISCRIMINANT_VALUE__{}_{}", to_pascal(enum_name), to_pascal(variant_name)),
            Span::call_site(),
        )
    }

    pub fn discriminant_check_struct(enum_name: &Ident, variant_name: &Ident) -> Ident {
        Ident::new(
            &format!("{}SpecifierVariant{}", enum_name, variant_name),
            Span::call_site(),
        )
    }

    pub fn discriminant_in_range(enum_name: &Ident, variant_name: &Ident) -> Ident {
        Ident::new(
            &format!("__BITFIELD_SPECIFIER_DISCRIMINANT_VALUE_OK__{}_{}", enum_name, variant_name),
            Span::call_site(),
        )
    }
}
