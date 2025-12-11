use proc_macro2::{Ident, TokenStream};
use quote::{format_ident, quote};
use syn::spanned::Spanned;
use syn::{parse_quote, AngleBracketedGenericArguments, Data, DataStruct, DeriveInput, Error, Expr, ExprLit, Field, Fields, FieldsNamed, GenericArgument, GenericParam, Generics, Lit, Meta, MetaNameValue, PathArguments, Type, TypeParam, TypePath};

pub fn derive_impl(input: DeriveInput) -> Result<TokenStream, Error> {
    let Data::Struct(DataStruct {
        fields: Fields::Named(FieldsNamed { named, .. }),
        ..
    }) = &input.data
    else {
        return Err(Error::new(input.span(), "input is not struct"))
    };

    let ident = &input.ident;
    let (impl_generics, ty_generics, where_clause) = &input.generics.split_for_impl();
    let mut where_clause = where_clause.cloned().unwrap_or_else(|| input.clone().generics.make_where_clause().to_owned());
    let debug_struct_fields = named.iter().map(|f| {
        debug_struct_field(f)
    });
    // トレイト境界の指定
    named.iter().for_each(|Field { ty, .. }| {
        if let Some(ty_generics) = is_generic_type_param(ty, &input.generics) {
            where_clause.predicates.push(parse_quote! {
                #ty_generics: std::fmt::Debug
            });
        }
    });

    Ok(quote! {
        impl #impl_generics std::fmt::Debug for #ident #ty_generics
        #where_clause
        {
            fn fmt(&self, fmt: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                fmt.debug_struct(stringify!(#ident))
                    #(#debug_struct_fields)*
                    .finish()
            }
        }
    })
}

fn debug_struct_field(Field { attrs, ident, .. }: &Field) -> TokenStream {
    // 最初に条件に合致したものをLitStrとして取り出す
    let fmt = attrs.iter().find_map(|attr| {
        match &attr.meta {
            Meta::NameValue(MetaNameValue { path, value: Expr::Lit(ExprLit { lit: Lit::Str(lit_str), .. }), .. }) => {
                if path.is_ident("debug") { Some(lit_str.value()) } else { None }
            },
            _ => None,
        }
    });

    if fmt.is_some() {
        quote! {
            .field(stringify!(#ident), &format_args!(#fmt, self.#ident))
        }
    } else {
        quote! {
            .field(stringify!(#ident), &self.#ident)
        }
    }
}

fn is_generic_type_param(ty: &Type, generics: &Generics) -> Option<Ident> {
    if let Type::Path(TypePath { path, .. }) = ty {
        if path.segments.len() == 1 {
            let segment = path.segments.first().unwrap();

            // ジェネリクスの型引数を持つ場合
            if !segment.arguments.is_empty() && segment.ident != "PhantomData" {
                if let PathArguments::AngleBracketed(AngleBracketedGenericArguments { args, .. }) = &segment.arguments {
                    // 型引数を再帰的にチェックする
                    return args.iter().find_map(|arg| {
                        match arg {
                            GenericArgument::Type(ty) => is_generic_type_param(ty, generics),
                            _ => None
                        }
                    })
                }
            // そうでない時
            } else {
                return generics.params.iter().find_map(|gp| {
                    match gp {
                        GenericParam::Type(TypeParam { ident, .. }) => if ident == &segment.ident { Some(ident.clone()) } else { None },
                        _ => None,
                    }
                })
            }
        // 関連型の場合
        } else if path.segments.len() == 2 {
            let ty_generics = &path.segments.first().unwrap().ident;
            let assoc_type = &path.segments.last().unwrap().ident;

            return generics.params.iter().find_map(|gp| {
                match gp {
                    GenericParam::Type(TypeParam { ident, .. }) => if ident == ty_generics {Some(format_ident!("{}::{}", ty_generics, assoc_type)) } else { None },
                    _ => None,
                }
            })
        }
    }
    None
}


