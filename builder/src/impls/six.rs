use proc_macro2::TokenStream;
use quote::quote;
use syn::{DeriveInput, Error};

pub fn derive_impl(input: &DeriveInput) -> Result<TokenStream, Error> {
    let ident = &input.ident;

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
