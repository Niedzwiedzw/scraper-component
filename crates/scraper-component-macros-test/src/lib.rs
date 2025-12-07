#[allow(dead_code)]
#[cfg(test)]
mod tests {
    use {
        scraper_component::{
            TryFromElement,
            anyhow::{self, Result},
            scraper::Html,
        },
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
    <div class="item">Meow</div>
</body>
"#;

            #[derive(Component)]
            struct ExampleStruct {
                #[component(selector = "div.item")]
                children: [String; 3],
            }

            #[test]
            fn test_parses() -> Result<()> {
                super::super::parsed::<ExampleStruct, _>(HTML, |element| {
                    const EXPECTED: &[&str] = &["Hello", "Hi", "Meow"];
                    anyhow::ensure!(element.children.eq(EXPECTED), "{EXPECTED:?} != {:?}", element.children);
                    Ok(())
                })
                .flatten()
            }
        }
    }
}
