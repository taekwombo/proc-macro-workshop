use proc_macro2::{Ident, Span, TokenStream};
use syn::{Attribute, Type, Lit, DataStruct, DeriveInput, Generics, GenericParam, TypePath, parse::Error};
use quote::quote;

const INVALID_DERIVE_INPUT: &str = "
Expected:
    
    #[derive(CustomDebug)]
    struct Name {
        ..
    }
";

const INVALID_DEBUG_ATTR: &str = r#"
Expected:

    #[debug = "..."]
"#;

macro_rules! err {
    ($msg:ident, $($i:tt)+) => {
        Err(Error::new_spanned($($i)+, $msg).to_compile_error())
    };
    ($($i:tt)+) => {
        Err(Error::new_spanned($($i)+, INVALID_DERIVE_INPUT).to_compile_error())
    }
}

pub fn get_data_struct(input: &DeriveInput) -> Result<&DataStruct, TokenStream> {
    use syn::Data;

    match input.data {
        Data::Struct(ref d) => Ok(d),
        _ => err!(input),
    }
}

fn find_debug_attr(attributes: &Vec<Attribute>) -> Result<Option<String>, TokenStream> {
    use syn::Meta;

    for attr in attributes {
        if !attr.path.is_ident("debug") {
            continue;
        }

        let meta = match attr.parse_meta() {
            Ok(m) => match m {
                Meta::NameValue(nv) => nv,
                _ => return err!(INVALID_DEBUG_ATTR, attr),
            },
            _ => continue,
        };

        return match meta.lit {
            Lit::Str(ref s) => Ok(Some(s.value())),
            _ => err!(INVALID_DEBUG_ATTR, attr),
        };
    }

    Ok(None)
}

pub struct FieldDebug<'a> {
    pub ident: &'a Ident,
    pub name: String,
    pub debug: Option<String>,
    pub ty: &'a Type,
}

pub fn get_fields_debug(data_struct: &DataStruct) -> Result<Vec<FieldDebug>, TokenStream> {
    use syn::Fields;

    let fields = match data_struct.fields {
        Fields::Named(ref n) => n,
        _ => return err!(&data_struct.fields),
    };

    let mut list = Vec::new();

    for f in &fields.named {
        if f.ident.is_none() {
            return err!(&data_struct.fields);
        }
        
        let ident = f.ident.as_ref().unwrap();
        let debug = match find_debug_attr(&f.attrs) {
            Ok(v) => v,
            Err(e) => return Err(e),
        };

        list.push(FieldDebug {
            ident,
            debug,
            name: ident.to_string(),
            ty: &f.ty,
        });
    }

    Ok(list)
}

// Input example: PhantomData<Option<Y>>
// Output example: Y
pub fn find_matching_generic_param_in_path<'a>(ty_path: &'a TypePath, name: &'a str) -> Option<&'a TypePath> {
    use syn::{GenericArgument, PathArguments};

    for seg in &ty_path.path.segments {
        if seg.ident == name {
            return Some(ty_path);
        }

        let path_args = match &seg.arguments {
            PathArguments::AngleBracketed(a) => a,
            _ => continue,
        };

        for arg in &path_args.args {
            let ty_arg = match arg {
                GenericArgument::Type(Type::Path(t)) => t,
                _ => continue,
            };

            if ty_arg.path.is_ident(name) {
                return Some(ty_arg);
            } 

            if let Some(r) = find_matching_generic_param_in_path(ty_arg, name) {
                return Some(r);
            };
        }
    }

    None
}

pub fn generate_where_clause(fields: &Vec<FieldDebug<'_>>, generics: &Generics) -> TokenStream {
    use syn::{token::Add, PathArguments, TraitBound, Path, TraitBoundModifier, TypeParamBound, WhereClause, PathSegment, PredicateType, punctuated::Punctuated};

    if generics.params.is_empty() {
        return TokenStream::new();
    }

    let mut predicates = generics.where_clause.clone().map(|p| p.predicates.clone()).unwrap_or_else(|| Punctuated::new());

    let debug_bounds: Punctuated<TypeParamBound, Add> = Punctuated::from_iter(vec![
        TypeParamBound::Trait(TraitBound {
            paren_token: None,
            modifier: TraitBoundModifier::None,
            lifetimes: None,
            path: Path {
                leading_colon: Some(Default::default()),
                segments: Punctuated::from_iter(vec![
                    PathSegment::from(Ident::new("std", Span::call_site())),
                    PathSegment::from(Ident::new("fmt", Span::call_site())),
                    PathSegment::from(Ident::new("Debug", Span::call_site())),
                ]),
            }
        })
    ]);

    let mut generic_params = generics
        .clone()
        .params
        .iter()
        .filter_map(|t| match t {
            GenericParam::Type(ty) => Some((ty.ident.to_string(), 0i32)),
            _ => None,
        })
        .collect::<Vec<_>>();

    let mut associated_types = Vec::new();

    for field in fields {
        let ty = match field.ty {
            Type::Path(t) => t,
            _ => continue,
        };

        let is_phantom = match ty.path.segments.first() {
            Some(ref p) => p.ident == "PhantomData",
            _ => false,
        };

        for gen in generic_params.iter_mut() {
            if let Some(p) = find_matching_generic_param_in_path(ty, &gen.0) {
                if is_phantom {
                    gen.1 += -1;
                    continue;
                }

                // If path length > 1 - then it is generic with its associated type.
                // In such case add new associated item.
                if p.path.segments.len() > 0 {
                    associated_types.push(p.clone());
                    gen.1 -= 1;
                } else {
                    gen.1 += 1;
                }
            }
        }
    }

    for (generic, cnt) in generic_params {
        if cnt < 0 {
            continue;
        }

        predicates.push(PredicateType {
            bounded_ty: TypePath {
                qself: None,
                path: Path {
                    leading_colon: None,
                    segments: Punctuated::from_iter(vec![PathSegment {
                        ident: Ident::new(&generic, Span::call_site()),
                        arguments: PathArguments::None,
                    } ]),
                },
            }.into(),
            lifetimes: None,
            colon_token: Default::default(),
            bounds: debug_bounds.clone(),
        }.into());
    }

    for ty in associated_types {
        predicates.push(PredicateType {
            bounded_ty: ty.into(),
            lifetimes: None,
            colon_token: Default::default(),
            bounds: debug_bounds.clone(),
        }.into());
    }

    if predicates.is_empty() {
        TokenStream::new()
    } else {
        quote! { where #predicates }
    }
}

pub fn strip_bounds_from_generics(generics: &Generics) -> Generics {
    let mut result = generics.clone();
    result.where_clause = None;

    result.params = result.params.into_iter().map(|mut g| match g {
        GenericParam::Type(ref mut i) => {
            i.bounds = syn::punctuated::Punctuated::new();
            g
        },
        _ => g,
    }).collect();

    result
}
