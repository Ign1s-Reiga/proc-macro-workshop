mod six;
mod seven;

use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::spanned::Spanned;
use syn::{AngleBracketedGenericArguments, Data, DataStruct, DeriveInput, Error, Field, Fields, FieldsNamed, Path, PathArguments, Type, TypePath};

enum FieldType<'a> {
    Required(&'a Type),
    Optional(&'a Type, &'a Type),
    // Repeated(&'a Type),
}

pub fn derive_impl(input: &DeriveInput) -> Result<TokenStream, Error> {
    // StructName
    let ident = &input.ident;
    let Data::Struct(DataStruct {
         fields: Fields::Named(FieldsNamed {
             named,
             ..
         }),
         ..
    }) = &input.data
    else {
        return Err(Error::new(input.span(), "only structs are supported"));
    };

    let mut builder_defaults: Vec<TokenStream> = vec![];
    let mut builder_fields: Vec<TokenStream> = vec![];
    let mut builder_setters: Vec<TokenStream> = vec![];
    let mut build_attrs: Vec<TokenStream> = vec![];

    named.iter().for_each(|field| {
        let field_ident = &field.ident;
        let build_err_msg = format!("{} must not be None", field_ident.clone().unwrap());
        let (builder_field, builder_setter, builder_attr) = match field_type(field) {
            FieldType::Required(ty) => {
                (quote! {
                    #field_ident: Option<#ty>,
                }, quote! {
                    pub fn #field_ident(&mut self, #field_ident: #ty) -> &mut Self {
                        self.#field_ident = Some(#field_ident);
                        self
                    }
                }, quote! {
                    #field_ident: self.#field_ident
                        .clone()
                        .ok_or_else(|| -> Box<dyn std::error::Error> { #build_err_msg.into() })?,
                })
            },
            FieldType::Optional(ty, ty_arg) => {
                (quote! {
                    #field_ident: #ty,
                }, quote! {
                    pub fn #field_ident(&mut self, #field_ident: #ty_arg) -> &mut Self {
                        self.#field_ident = Some(#field_ident);
                        self
                    }
                }, quote! {
                    #field_ident: self.#field_ident.clone(),
                })
            },
        };

        builder_defaults.push(quote! {
            #field_ident: None,
        });
        builder_fields.push(builder_field);
        builder_setters.push(builder_setter);
        build_attrs.push(builder_attr);
    });

    let builder_ident = format_ident!("{}Builder", ident);

    Ok(quote! {
        // ビルダー
        pub struct #builder_ident {
            #(#builder_fields)*
        }

        impl #builder_ident {
            // トレイトは直接指定
            fn build(self) -> Result<Command, Box<dyn std::error::Error>> {
                Ok(Command {
                    // Resultに変換・NoneだったらErrを返す
                    #(#build_attrs)*
                })
            }

            #(#builder_setters)*
        }

        impl #ident {
            fn builder() -> #builder_ident {
                #builder_ident {
                    #(#builder_defaults)*
                }
            }
        }
    })
}

// フィールドのタイプ
fn field_type(field: &Field) -> FieldType<'_> {
    let ty = &field.ty;
    match ty {
        Type::Path(TypePath { path, .. }) => {
            if let Some(ident) = path.get_ident() && ident.to_string().starts_with("Option") {
                FieldType::Optional(ty, get_type_argument(path).unwrap())
            } else {
                FieldType::Required(ty)
            }
        },
        bad => FieldType::Required(&bad)
    }
}

// Option<T>のTを取得
fn get_type_argument(path: &Path) -> Option<&Type> {
    let segment = path.segments.last()?;
    if let PathArguments::AngleBracketed(AngleBracketedGenericArguments { args, .. }) = &segment.arguments {
        if let Some(syn::GenericArgument::Type(ty)) = args.first() {
            return Some(ty);
        }
    }
    None
}
