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
                scraper_component::{
                    Single,
                    anyhow::{self, Result},
                },
            };

            #[rustfmt::skip]
            const HTML: &str = r#"
<!DOCTYPE html>
<body>
    <div class="item">Hello</div>
    <div class="item">Hi</div>
    <div class="item" id="something">Meow</div>
</body>
"#;
            #[derive(Component, Debug, Clone)]
            struct OptionalIdChild {
                #[component(map = "scraper_component::attribute::id_opt")]
                id: Single<Option<String>>,
            }
            #[derive(Component, Debug, Clone)]
            struct RequiredIdChild {
                #[component(map = "scraper_component::attribute::id")]
                id: Single<String>,
            }

            #[derive(Component, Clone)]
            struct ExampleStruct {
                #[component(selector = "div.item")]
                children: [String; 3],
                #[component(selector = "div.item")]
                children_2: [OptionalIdChild; 3],
            }

            #[test]
            fn test_parses() -> Result<()> {
                super::super::parsed::<ExampleStruct, _>(HTML, |element| {
                    let expected: &[_] = &[[None], [None], [Some("something".to_string())]];
                    anyhow::ensure!(
                        element.children_2.clone().map(|c| c.id).eq(expected),
                        "{expected:?} != {:?}",
                        element.children_2
                    );
                    Ok(())
                })
                .flatten()
            }
        }
    }
}
