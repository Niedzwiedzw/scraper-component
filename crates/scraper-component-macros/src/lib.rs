use {
    anyhow::{Context, Result},
    darling::FromDeriveInput,
    proc_macro::{self, TokenStream},
    quote::quote,
    syn::{DeriveInput, parse_macro_input},
    tap::{Pipe, Tap, TapFallible},
};

mod component;

trait AnyhowExt<T> {
    fn for_anyhow(self) -> Result<T>;
}

impl<T> AnyhowExt<T> for darling::Result<T> {
    fn for_anyhow(self) -> Result<T> {
        self.map_err(|e| anyhow::anyhow!("DARLING:\n{e:?}"))
    }
}

// Struct to parse derive input attributes
#[derive(FromDeriveInput, Clone, Debug)]
#[darling(attributes(component), supports(struct_any, enum_any))]
struct ComponentInput {
    ident: syn::Ident,
    generics: syn::Generics,
}

#[proc_macro_derive(Component, attributes(component))]
pub fn component_macro(input: TokenStream) -> TokenStream {
    // Parse input with darling
    {
        let input = input.clone().tap_dbg(|ts| {
            #[cfg(debug_assertions)]
            {
                ts.clone()
                    .pipe(proc_macro2::TokenStream::from)
                    .pipe_ref(utils::format_macro_output)
                    .pipe(utils::syntax_highlighting)
                    .pipe(|ts| {
                        use std::io::Write;
                        println!("{ts}");
                        std::io::stdout().flush().unwrap();
                    })
            }
        });
        parse_macro_input!(input as DeriveInput)
    }
    .pipe_ref(|input| {
        ComponentInput::from_derive_input(&input.clone())
            .for_anyhow()
            .map(|component| (input, component))
    })
    .context("parsing input")
    .and_then(|(input, component_input)| component::derive_component_impl(input, component_input))
    .with_context(|| format!("parsing:\n{input}"))
    .tap_ok_dbg(|ts| {
        #[cfg(debug_assertions)]
        {
            ts.pipe(utils::format_macro_output)
                .pipe(utils::syntax_highlighting)
                .pipe(|ts| {
                    use std::io::Write;
                    println!("{ts}");
                    std::io::stdout().flush().unwrap();
                })
        }
    })
    .tap_err(|err| eprintln!("ERROR:\n{err:?}"))
    .map(TokenStream::from)
    .unwrap_or_else(|e| panic!("scraper-component proc macro failed\nreason\n{e:?}"))
}
mod utils;
