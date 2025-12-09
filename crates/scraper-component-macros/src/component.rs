use {
    super::*,
    crate::component::struct_field_kind::StructFieldKind,
    anyhow::{Result, anyhow},
    darling::FromField,
    syn::{DataStruct, Path},
};

mod struct_field_kind {
    use syn::Ident;
    #[derive(Debug, Clone)]
    pub enum StructFieldKind {
        Named(syn::Ident),
        Index(syn::Index),
    }

    impl StructFieldKind {
        pub fn index(original: &Ident, index: usize) -> Self {
            Self::Index(syn::Index {
                index: index as _,
                span: original.span(),
            })
        }
    }

    impl quote::ToTokens for StructFieldKind {
        fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
            match self {
                StructFieldKind::Named(ident) => ident.to_tokens(tokens),
                StructFieldKind::Index(index) => index.to_tokens(tokens),
            }
        }
    }
    impl std::fmt::Display for StructFieldKind {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            match self {
                StructFieldKind::Named(ident) => ident.to_string().fmt(f),
                StructFieldKind::Index(index) => index.index.to_string().fmt(f),
            }
        }
    }
}

fn default_map_path() -> syn::Path {
    syn::parse_quote!(::scraper_component::try_from_element)
}

/// Struct to parse field attributes
#[derive(FromField, Debug)]
#[darling(attributes(component))]
struct ComponentField {
    ident: Option<syn::Ident>,
    #[allow(dead_code)]
    ty: syn::Type,
    #[darling(default)]
    selector: Option<String>,
    #[darling(default = "default_map_path")]
    map: Path,
    #[darling(default)]
    many: bool
    // Field name
    // #[darling(default)]
    // skip: bool, // #[bimber(skip)] to skip fields
}

// #[derive(FromVariant)]
// #[darling(attributes(bimber))]
// struct ComponentVariant {
//     #[allow(dead_code)]
//     ident: syn::Ident,
//     fields: darling::ast::Fields<ComponentField>,
// }

// pub enum OriginalFieldKind {
//     StructField { field_name: StructFieldKind },
//     EnumVariant { variant_name: Ident },
// }

// impl std::fmt::Display for OriginalFieldKind {
//     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//         match self {
//             OriginalFieldKind::StructField { field_name } => field_name.to_string().fmt(f),
//             OriginalFieldKind::EnumVariant { variant_name } => variant_name.to_string().fmt(f),
//         }
//     }
// }

pub fn derive_component_impl(input: &DeriveInput, ComponentInput { ident: struct_name, generics }: ComponentInput) -> Result<proc_macro2::TokenStream> {
    match &input.data {
        syn::Data::Struct(DataStruct { fields, .. }) => fields
            .iter()
            .map(|f| {
                ComponentField::from_field(f)
                    .for_anyhow()
                    .with_context(|| format!("parsing field: {f:?}"))
            })
            .collect::<Result<Vec<_>>>()
            .context("collecting fields")
            .map(|fields| {
                fields
                    .iter()
                    .enumerate()
                    .map(
                        |(
                            idx,
                            component_field @ ComponentField {
                                ident,
                                ty: _,
                                selector: _,
                                map: _,
                                many: _
                            },
                        )| {
                            ident
                                .as_ref()
                                .map(|i| StructFieldKind::Named(i.clone()))
                                .unwrap_or(StructFieldKind::index(&input.ident, idx))
                                .pipe(|field_kind| (field_kind, component_field))
                        },
                    )
                    .map(
                        |(
                            kind,
                            ComponentField {
                                ident: _,
                                ty,
                                selector,
                                map,
                                many
                            },
                        )| {
                            let selector = selector.as_ref();

                            let selector_str = selector.map(|s| s.to_string());
                            let field_name = kind.to_string();
                            let struct_name = struct_name.to_string();
                            // VALIDATE AT COMPILE TIME
                            let _selector = selector
                                .map(|selector| scraper::Selector::parse(selector))
                                .transpose()
                                .map_err(|e| anyhow::anyhow!("{e:?}"))
                                .with_context(|| format!("invalid selector for field '{field_name}': '{selector:?}'"))
                                .unwrap();

                            let define_selector = match selector {
                                Some(selector) => {
                                    quote! {
                                        Some(::scraper_component::scraper::Selector::parse(#selector).expect("validated at compile time"))
                                    }
                                }
                                None => quote! {
                                    None
                                },
                            };
                            let selector_str = selector_str.unwrap_or_else(|| "<no-selector>".into());

                            let perform_parse = match many {
                                true => quote! {
                                    <#ty as ::scraper_component::TryCollectFrom<_>>::try_collect(mapped)
                                        .with_context(|| format!("reading {}::{} (selector: {}) from:\n{}", #struct_name, #field_name, #selector_str, ___element.html()))
                                    
                                },
                                false => quote! {
                                    <[#ty; 1] as ::scraper_component::TryCollectFrom<_>>::try_collect(mapped)
                                        .with_context(|| format!("reading {}::{} (selector: {}) from:\n{}", #struct_name, #field_name, #selector_str, ___element.html()))
                                        .map(|[v]| v)
                                },
                            };
                            (
                                kind.clone(),
                                quote::quote! {
                                    let #kind = {
                                        use ::scraper_component::{anyhow::{Result, Context, anyhow}, scraper::Selector};
                                        thread_local! {
                                            static SELECTOR: Option<::scraper_component::scraper::Selector> =
                                                #define_selector;    
                                        }
                                        SELECTOR.with(|selector| {
                                            let select = selector.as_ref().map(|selector| {
                                                (Box::new(___element.select(selector)) as Box<dyn Iterator<Item = _>>)
                                            })
                                            .unwrap_or_else(|| Box::new(std::iter::once(___element)));
                                            let mapped = select.map(#map);
                                            #perform_parse    
                                        })
                                    }?;
                                },
                            )
                        },
                    )
                    .collect::<Vec<_>>()
                    .pipe(|fields| {
                        let field_impls = fields.iter().map(|(_, f)| f);
                        let (_impl_generics, type_generics, where_clause) = generics.split_for_impl();
                        let field_names = fields.iter().map(|(f, _)| f);
                        quote! {
                            impl <'document> ::scraper_component::TryFromElement<'document> for #struct_name
                            #type_generics
                            #where_clause{
                                fn try_from_element(___element: ::scraper_component::scraper::ElementRef<'document>)
                                    ->
                                ::scraper_component::anyhow::Result<Self> {
                                    #(#field_impls)*

                                    Ok(Self {
                                        #(#field_names,)*
                                    })
                                }
                            }
                        }
                    })
            }),
        syn::Data::Enum(_data_enum) => Err(anyhow!("enums not supported")),
        syn::Data::Union(_data_union) => Err(anyhow!("unions are not supported")),
    }
    .context("deriving component")
}
