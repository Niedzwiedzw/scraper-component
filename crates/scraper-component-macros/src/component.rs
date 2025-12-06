use {
    super::*,
    crate::component::struct_field_kind::StructFieldKind,
    anyhow::{Result, anyhow},
    darling::FromField,
    syn::DataStruct,
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

/// Struct to parse field attributes
#[derive(FromField, Debug)]
#[darling(attributes(component))]
struct ComponentField {
    ident: Option<syn::Ident>,
    #[allow(dead_code)]
    ty: syn::Type,
    selector: String,
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
        syn::Data::Struct(DataStruct { fields, .. }) => {
            fields
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
                    .map(|(idx, component_field @ ComponentField { ident, ty: _, selector: _ })| {
                        ident
                            .as_ref()
                            .map(|i| StructFieldKind::Named(i.clone()))
                            .unwrap_or(StructFieldKind::index(&input.ident, idx))
                            .pipe(|field_kind| (field_kind, component_field))
                    })
                    .map(|(kind, ComponentField { ident: _, ty: _, selector })| {
                        
                        let selector_str = selector.to_string();
                        let field_name = kind.to_string();
                        let struct_name = struct_name.to_string();
                        (kind.clone(), quote::quote! {
                            let #kind = {
                                use ::scraper_component::{anyhow::{Result, Context, anyhow}, scraper::Selector};
                                Selector::parse(#selector)
                                    .map_err(|e| anyhow!("{e:?}"))
                                    .with_context(|| format!("invalid selector: '{}'", #selector_str))
                                    .and_then(|sel| {
                                        let out = ___element
                                            .select(&sel)
                                            .map(::scraper_component::TryFromElement::try_from_element);
                                        <_ as ::scraper_component::TryCollectFrom<_>>::try_collect(out)
                                    })
                                    .with_context(|| format!("reading {}::{}", #struct_name, #field_name))
                            }?;
                        })
                    })
                    .collect::<Vec<_>>()
                    .pipe(|fields| {
                        let field_impls = fields.iter().map(|(_, f)| f);
                        let (_impl_generics, type_generics, where_clause) = generics.split_for_impl();
                        let field_names = fields.iter().map(|(f, _)| f);
                        quote! {
                            impl <'document> ::scraper_component::TryFromElement<'document> for #struct_name #type_generics #where_clause{
                                fn try_from_element(___element: ::scraper_component::scraper::ElementRef<'document>) -> ::scraper_component::anyhow::Result<Self> {
                                    #(#field_impls)*

                                    Ok(Self {
                                        #(#field_names,)*
                                    })
                                }
                            }
                        }
                    })
                })
        }
        syn::Data::Enum(_data_enum) => Err(anyhow!("enums not supported")),
        syn::Data::Union(_data_union) => Err(anyhow!("unions are not supported")),
    }
    .context("deriving component")
}
