#[allow(dead_code)]
#[cfg(test)]
mod tests {
    use {
        scraper_component::{TryFromElement, anyhow::Result, scraper::Html},
        tap::Pipe,
    };

    fn parsed<'document, T, U>(fragment: &'static str, with: impl FnOnce(T) -> U) -> Result<U>
    where
        T: for<'a> TryFromElement<'a> + Sized + 'document,
        U: Sized + 'static,
    {
        Html::parse_fragment(fragment).pipe_ref(|html| {
            html.root_element()
                .pipe(TryFromElement::try_from_element)
                .map(with)
        })
    }
    pub mod impl_struct {
        use scraper_component::Component;

        pub mod three_texts {
            use {
                super::*,
                scraper_component::anyhow::{self, Result},
            };

            #[rustfmt::skip]
            const HTML: &str = r#"
<!DOCTYPE html>
<body>
    <div class="item">Hello</div>
    <div class="item">Hi</div>
    <div class="item" id="cat">Meow</div>
</body>
"#;
            #[derive(Component, Debug, Clone, PartialEq)]
            struct OptionalIdChild {
                #[component]
                text: String,
                // id is optional
                #[component(map = "scraper_component::attribute::id_opt")]
                id: Option<String>,
            }

            #[derive(Component, Clone, PartialEq)]
            struct ExampleStruct {
                ///  inlines entire text from matching elements
                #[component(selector = "div.item", many)]
                children_simple: [String; 3],
                /// parses nested component for more granular parsing
                #[component(selector = "div.item", many)]
                children_via_struct: [OptionalIdChild; 3],
            }

            #[test]
            fn test_parses() -> Result<()> {
                super::super::parsed::<ExampleStruct, _>(HTML, |element| {
                    anyhow::ensure!(
                        element
                            .children_simple
                            .eq(&["Hello", "Hi", "Meow"].map(ToOwned::to_owned)),
                    );

                    anyhow::ensure!(element.children_via_struct.clone().eq(&[
                        OptionalIdChild {
                            text: "Hello".into(),
                            id: None
                        },
                        OptionalIdChild { text: "Hi".into(), id: None },
                        OptionalIdChild {
                            text: "Meow".into(),
                            id: Some("cat".into())
                        },
                    ]),);
                    Ok(())
                })
                .flatten()
            }
        }
    }
}
