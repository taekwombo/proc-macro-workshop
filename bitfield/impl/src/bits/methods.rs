use proc_macro2::{Ident, Span, TokenStream};
use syn::LitInt;

pub fn names(field_name: &Ident) -> (Ident, Ident) {
    (
        Ident::new(&format!("get_{}", field_name), Span::call_site()),
        Ident::new(&format!("set_{}", field_name), Span::call_site()),
    )
}

fn get_upper_size_bound(size: u8) -> u8 {
    match size {
        1..=8 => 8,
        9..=16 => 16,
        17..=32 => 32,
        33..=64 => 64,
        _ => unreachable!(),
    }
}

pub fn matching_value_type(size: u8) -> (LitInt, Ident) {
    let ty_size = get_upper_size_bound(size);

    (
        LitInt::new(&format!("0_u{}", ty_size), Span::call_site()),
        Ident::new(&format!("u{}", ty_size), Span::call_site())
    )
}

#[derive(Debug, PartialEq)]
pub struct Byte {
    start: usize,
    pub len: usize,
    index: usize,
}

pub fn field_bytes(prev: usize, bits: usize) -> Vec<Byte> {
    let mut bytes: Vec<Byte> = Vec::new();

    if bits == 0 {
        panic!();
    }

    let min = prev;
    let max = prev + bits;
    // Bit index of the first bit of current byte.
    let mut byte_start = prev;

    for i in min..=max {
        if i != min && i % 8 == 0 {
            bytes.push(Byte {
                start: byte_start % 8,
                len: i - byte_start,
                index: (i - 1) / 8,
            });
            byte_start = i;
            continue;
        }

        if i == max {
            bytes.push(Byte {
                start: byte_start % 8,
                len: i - byte_start,
                index: (i - 1) / 8,
            });
        }
    }

    bytes
}

fn get_byte_mask(start: usize, len: usize, keep_byte_bits: bool) -> LitInt {
    let (out, content) = if keep_byte_bits { ("0", "1") } else { ("1", "0") };

    let mut repr = out.repeat(start);

    repr.push_str(&content.repeat(len));

    if start + len != 8 {
        repr.push_str(&out.repeat(8 - start - len));
    }

    assert_eq!(repr.len(), 8);

    LitInt::new(
        &format!("0b{}u8", repr),
        Span::call_site(),
    )
}

pub fn get_data_byte(shift: usize, ty: &Ident, byte: &Byte) -> TokenStream {
    use quote::quote;

    let Byte { start, len, index } = byte;

    let mut value = quote! { (self.data[#index] as #ty) };

    // Apply mask for cases like XXBB_XXXX.
    if *len != 8 {
        let mask = get_byte_mask(*start, *len, true);
        value = quote! { (#value & #mask) };
    }

    // Apply right shift for cases like BBBX_XXXX or XXXB_BXXX.
    if start + len != 8 {
        let shift_by = 8 - len - start;
        value = quote!{ (#value >> #shift_by) };
    }

    // Apply left shift so the bits end up in correct position.
    if shift != 0 {
        value = quote! { (#value << #shift) };
    }

    value
}

pub fn set_data_byte(shift: usize, byte: &Byte) -> TokenStream {
    use quote::quote;

    let Byte { start, len, index } = byte;

    // Discard already written bytes.
    let mut value = if shift > 0 { quote! { (value >> #shift) } } else { quote! { value } };

    // Apply left shift for cases like BBBB_XXXX.
    if start + len != 8 {
        let left_shift = 8 - start - len;
        value = quote! {
            (#value << #left_shift)
        };
    }

    value = quote!{ (#value as u8) };

    let mut data_byte = quote! { self.data[#index] };

    if *len != 8 {
        let value_mask = get_byte_mask(*start, *len, true);
        let data_byte_mask = get_byte_mask(*start, *len, false);

        value = quote! {
            (#value & #value_mask)
        };

        data_byte = quote! {
            (#data_byte & #data_byte_mask)
        };
    }

    quote! { self.data[#index] = (#data_byte) | (#value); }
}

#[cfg(test)]
mod test {
    use super::*;

    macro_rules! test_bytes {
        ($fn_call:expr, $len:literal, [$({ start: $s:literal, index: $i:literal, len: $l:literal },)+]) => {
            {
                let bytes = $fn_call;
                let mut index = 0usize;
                assert_eq!(bytes.len(), $len);
                $(
                    assert_eq!(bytes[index].start, $s);
                    assert_eq!(bytes[index].index, $i);
                    assert_eq!(bytes[index].len, $l);
                    index += 1;
                )+
                assert_eq!($len, index);
            }
        }
    }

    #[test]
    fn test_get_field_bytes() {
        test_bytes!(field_bytes(0, 1), 1, [
            { start: 0, index: 0, len: 1 },
        ]);

        test_bytes!(field_bytes(7, 1), 1, [
            { start: 7, index: 0, len: 1 },
        ]);

        test_bytes!(field_bytes(15, 1), 1, [
            { start: 7, index: 1, len: 1 },
        ]);

        test_bytes!(field_bytes(0, 12), 2, [
            { start: 0, index: 0, len: 8 },
            { start: 0, index: 1, len: 4 },
        ]);

        test_bytes!(field_bytes(7, 14), 3, [
            { start: 7, index: 0, len: 1 },
            { start: 0, index: 1, len: 8 },
            { start: 0, index: 2, len: 5 },
        ]);

        test_bytes!(field_bytes(15, 2), 2, [
            { start: 7, index: 1, len: 1 },
            { start: 0, index: 2, len: 1 },
        ]);

        test_bytes!(field_bytes(80, 20), 3, [
            { start: 0, index: 10, len: 8 },
            { start: 0, index: 11, len: 8 },
            { start: 0, index: 12, len: 4 },
        ]);
    }
}
