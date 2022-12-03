use proc_macro2::{Ident, Span};

fn to_pacal(ident: &Ident) -> String {
    let mut s = String::new();

    let input = ident.to_string();
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

pub fn size_const(ident: &Ident) -> Ident {
    Ident::new(
        &format!("__BITFIELD_SIZE__{}", to_pacal(ident)),
        Span::call_site(),
    )
}

pub fn size_const_mod(ident: &Ident) -> Ident {
    Ident::new(
        &format!("__BITFIELD_SIZE_MOD__{}", to_pacal(ident)),
        Span::call_site(),
    )
}
