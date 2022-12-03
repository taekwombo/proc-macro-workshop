#![feature(proc_macro_span)]

use proc_macro::TokenStream;
use proc_macro2::{TokenStream as TokenStream2, Ident};
use syn::{DataStruct, DeriveInput, Field, Fields, Type};

// Get syn::DataStruct out of syn::Data..
// If Builder macro was applied to Union or Struct then returns TokenStream with error.
fn get_struct_data(derive_input: &DeriveInput) -> Result<&DataStruct, TokenStream> {
    use proc_macro2::Span;
    use quote::quote_spanned;
    use syn::token::{Enum, Union};
    use syn::{Data, DataEnum, DataUnion};

    let DeriveInput { ident, data, .. } = derive_input;

    let join_spans = |b: Span| -> Span {
        let a = ident.span();
        let joined = a.join(b);

        joined.unwrap_or(a)
    };

    let span = match data {
        Data::Struct(d) => return Ok(d),
        Data::Union(DataUnion {
            union_token: Union { span },
            ..
        })
        | Data::Enum(DataEnum {
            enum_token: Enum { span },
            ..
        }) => span,
    };

    Err(quote_spanned! {
        join_spans(*span) => compile_error!("Builder macro supports struct only.")
    }
    .into())
}

fn create_builder_ident(target_ident: &Ident) -> Ident {
    Ident::new(
        &format!("{}Builder", target_ident),
        proc_macro2::Span::call_site(),
    )
}

// Get list of struct fields.
// If Builder macro was applied to unit or tuple struct then returns TokenStream with error.
fn get_target_fields<'a>(
    target_ident: &'a Ident,
    fields: &'a Fields,
) -> Result<Vec<&'a Field>, TokenStream> {
    use quote::quote_spanned;

    match fields {
        Fields::Named(f) => Ok(f.named.iter().collect::<Vec<_>>()),
        _ => Err(quote_spanned! {
            target_ident.span() => compile_error!("Builder macro supports only structs.")
        }
        .into()),
    }
}

// Get list of builder fields.
//
// Returns (*builder fields definition*, *builder fields new*, *builder_methods*, *builder build*).
//
// struct Builder {
//     {builder_field_name: builder_field_type}
//     ..
// }
//
// impl Builder {
//     fn new() -> Self {
//         Self {
//             {builder_field_name: builder_field_type}
//             ..
//         }
//     }
//
//     fn {builder_field_name} (&mut self, val: {builder_method_type}) -> &mut Self;
//
//     fn build(&mut self) -> {target} {
//         Self {
//             {builder_field_name: builder_field_build}
//             ..
//         }
//     }
//     ..
// }
fn get_builder_fields(
    target_fields: &[&Field],
) -> (
    Vec<TokenStream2>,
    Vec<TokenStream2>,
    Vec<TokenStream2>,
    Vec<TokenStream2>,
) {
    use quote::quote;
    use syn::Attribute;

    macro_rules! get_or_none {
        ($t:expr, $($arm:tt)+) => {
            match $t {
                $($arm)*,
                _ => return None,
            }
        }
    }

    fn get_inner_type<'a>(ty: &'a Type, outter: &'static str) -> Option<&'a Type> {
        use syn::{GenericArgument, PathArguments};

        let path = get_or_none!(ty, Type::Path(p) => p);
        let first = path.path.segments.iter().next();
        let seg = get_or_none!(first, Some(s) => s);

        if seg.ident != outter {
            return None;
        }

        let type_arg = get_or_none!(seg.arguments, PathArguments::AngleBracketed(ref i) => i);

        if type_arg.args.len() > 1 {
            return None;
        }

        let inner = get_or_none!(type_arg.args.iter().next(), Some(p) => p);

        Some(get_or_none!(inner, GenericArgument::Type(ref ty) => ty))
    }

    fn get_each_attr(attrs: &Vec<Attribute>) -> Result<Option<Ident>, TokenStream2> {
        use syn::{parse, Lit, Meta, NestedMeta};

        for attr in attrs {
            if !attr.path.is_ident("builder") {
                continue;
            }

            let meta = attr.parse_meta();

            if meta.is_err() {
                return Err(parse::Error::new_spanned(
                    attr,
                    "Unrecognized argument to builder attribute",
                )
                .to_compile_error());
            }

            let meta = meta.unwrap();

            let list = match meta {
                Meta::List(n) => n,
                _ => {
                    return Err(parse::Error::new_spanned(
                        meta,
                        "Unrecognized argument to builder attribute",
                    )
                    .to_compile_error())
                }
            };

            if list.nested.len() != 1 {
                return Err(parse::Error::new_spanned(
                    list,
                    "Unrecognized argument to builder attribute",
                )
                .to_compile_error());
            }

            // Now, we are sure that attribute is something like "#[builder(...)]".
            let nested_meta = match list.nested.iter().next().unwrap() {
                NestedMeta::Meta(m) => match m {
                    Meta::NameValue(v) => v,
                    un => {
                        return Err(parse::Error::new_spanned(
                            un,
                            "Unrecognized argument to builder attribute",
                        )
                        .to_compile_error())
                    }
                },
                un => {
                    return Err(parse::Error::new_spanned(
                        un,
                        "Unrecognized argument to builder attribute",
                    )
                    .to_compile_error())
                }
            };

            if !nested_meta.path.is_ident("each") {
                return Err(parse::Error::new_spanned(
                    &list,
                    r#"expected `builder(each = "...")`"#,
                )
                .to_compile_error());
            }

            let name = match nested_meta.lit {
                Lit::Str(ref s) => s,
                _ => {
                    return Err(parse::Error::new_spanned(
                        &list,
                        r#"expected `builder(each = "...")`"#,
                    )
                    .to_compile_error())
                }
            };

            return Ok(Some(Ident::new(&name.value(), name.span())));
        }

        Ok(None)
    }

    target_fields
        .iter()
        .map(|f| {
            let ident = f
                .ident
                .clone()
                .expect("Fields should be named at this point.");
            let t_opt = quote! { ::std::option::Option };

            if let Some(opt_inner) = get_inner_type(&f.ty, "Option") {
                return (
                    quote! { #ident: #t_opt<#opt_inner> },
                    quote! { #ident: #t_opt::None },
                    quote! {
                        fn #ident (&mut self, value: #opt_inner) -> &mut Self {
                            self.#ident.replace(value);
                            self
                        }
                    },
                    quote! { #ident: self.#ident.take() },
                );
            }

            if let Some(vec_inner) = get_inner_type(&f.ty, "Vec") {
                let t_vec = quote! { ::std::vec::Vec };
                let each_attr = get_each_attr(&f.attrs);

                let same_each_as_field = match each_attr {
                    Ok(Some(ref each_ident)) => *each_ident == ident,
                    _ => false,
                };

                let methods = if same_each_as_field {
                    quote! {
                        fn #ident (&mut self, value: #vec_inner) -> &mut Self {
                            self.#ident.push(value);
                            self
                        }
                    }
                } else {
                    let each_output = match each_attr {
                        Err(e) => e,
                        Ok(None) => quote! {},
                        Ok(Some(lit)) => quote! {
                            fn #lit (&mut self, value: #vec_inner) -> &mut Self {
                                self.#ident.push(value);
                                self
                            }
                        },
                    };

                    quote! {
                        #each_output

                        fn #ident (&mut self, mut value: #t_vec<#vec_inner>) -> &mut Self {
                            self.#ident.append(&mut value);
                            self
                        }
                    }
                };

                return (
                    quote! { #ident: #t_vec<#vec_inner> },
                    quote! { #ident: #t_vec::new() },
                    methods,
                    quote! {
                        #ident: {
                            let mut val = #t_vec::new();
                            ::std::mem::swap(&mut self.#ident, &mut val);
                            val
                        }
                    },
                );
            }

            let ty = &f.ty;
            (
                quote! { #ident: #t_opt<#ty> },
                quote! { #ident: #t_opt::None },
                quote! {
                    fn #ident (&mut self, value: #ty) -> &mut Self {
                        self.#ident.replace(value);
                        self
                    }
                },
                quote! {
                    #ident: match self.#ident.take() {
                        #t_opt::Some(v) => v,
                        #t_opt::None => return #t_opt::None,
                    }
                },
            )
        })
        .fold(
            (Vec::new(), Vec::new(), Vec::new(), Vec::new()),
            |mut acc, (def, new, met, build)| {
                acc.0.push(def);
                acc.1.push(new);
                acc.2.push(met);
                acc.3.push(build);
                acc
            },
        )
}

#[proc_macro_derive(Builder, attributes(builder))]
pub fn derive(input: TokenStream) -> TokenStream {
    use quote::quote;
    use syn::parse_macro_input;

    let derive_input = parse_macro_input!(input as DeriveInput);
    let data_struct = match get_struct_data(&derive_input) {
        Ok(d) => d,
        Err(e) => return e,
    };

    let DeriveInput {
        ident: target_ident,
        vis: target_vis,
        ..
    } = &derive_input;

    let target_fields = match get_target_fields(target_ident, &data_struct.fields) {
        Ok(f) => f,
        Err(e) => return e,
    };

    let builder_ident = create_builder_ident(target_ident);
    let (builder_def, builder_new, builder_met, builder_build) = get_builder_fields(&target_fields);

    // Struct builder definition.
    let builder_def = quote! {
        #target_vis struct #builder_ident {
            #(#builder_def,)*
        }
    };

    // Struct builder impl block.
    let builder_impl = quote! {
        impl #builder_ident {
            fn new() -> Self {
                Self {
                    #(#builder_new,)*
                }
            }

            #(#builder_met)*

            fn build(&mut self) -> ::std::option::Option<#target_ident> {
                Some(#target_ident {
                    #(#builder_build,)*
                })
            }
        }
    };

    // The `build` method implementation on marked struct.
    let target_bulid_impl = quote! {
        impl #target_ident {
            #target_vis fn builder() -> #builder_ident {
                #builder_ident::new()
            }
        }
    };

    quote! {
        #target_bulid_impl
        #builder_def
        #builder_impl
    }
    .into()
}
