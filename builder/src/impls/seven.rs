use proc_macro2::{Ident, TokenStream};
use quote::{format_ident, quote};
use syn::{Attribute, Data, DataStruct, DeriveInput, Error, Field, Fields, FieldsNamed, LitStr, Meta, MetaList};
use syn::spanned::Spanned;

pub fn derive_impl(input: &DeriveInput) -> Result<TokenStream, Error> {
    let ident = &input.ident;
    let Data::Struct(DataStruct { fields, .. }) = &input.data
    else {
        return Err(Error::new(input.span(), "data is not struct"))
    };
    let Fields::Named(FieldsNamed { named, .. }) = fields
    else {
        return Err(Error::new(input.span(), "fields is not named "))
    };

    for Field { attrs, .. } in named {
        if let Some(attrs) = each_attr(attrs) {
            if let Ok(value) = attrs {

            }
        } else {
            continue;
        }
    }

    Ok(quote! {
        // ビルダーパターン
        pub struct CommandBuilder {
            executable: Option<String>,
            args: Option<Vec<String>>,
            env: Option<Vec<String>>,
            current_dir: Option<String>,
        }

        impl CommandBuilder {
            fn build(self) -> Result<Command, Box<dyn std::error::Error>> {
                Ok(Command {
                    executable: self.executable.ok_or_else(|| "executable must not be None")?,
                    args: self.args.ok_or_else(|| "args must not be None")?,
                    env: self.env.ok_or_else(|| "env must not be None")?,
                    current_dir: self.current_dir,
                })
            }

            fn executable(mut self, executable: String) -> Self {
                self.executable = Some(executable);
                self
            }

            fn args(mut self, args: Vec<String>) -> Self {
                self.args = Some(args);
                self
            }

            fn env(mut self, env: Vec<String>) -> Self {
                self.env = Some(env);
                self
            }

            fn current_dir(mut self, current_dir: String) -> Self {
                self.current_dir = Some(current_dir);
                self
            }
        }

        impl #ident {
            fn builder() -> CommandBuilder {
                CommandBuilder {
                    executable: None,
                    args: None,
                    env: None,
                    current_dir: None,
                }
            }
        }
    })
}

fn each_attr(attrs: &Vec<Attribute>) -> Option<Result<Ident, Error>> {
    let mut attr_builder: Option<Result<Ident, Error>> = None;
    for attr in attrs {
        let Meta::List(MetaList { path, .. }) = &attr.meta
        else {
            continue;
        };

        if path.is_ident("builder") {
            let _ = attr.parse_nested_meta(|meta| {
                if meta.path.is_ident("each") {
                    attr_builder = Some(Ok(format_ident!("{}", meta.value()?.parse::<LitStr>()?.value())));
                } else {
                    attr_builder = Some(Err(Error::new(attr.span(), "expected `builder(each = \"...\")`")));
                }
                Ok(())
            });
        }
    }

    attr_builder
}
