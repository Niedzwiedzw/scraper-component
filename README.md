# scraper-component

A Rust library providing a procedural macro (`#[derive(Component)]`) and helpers for declaratively parsing HTML elements using the `scraper` crate.

## Features
- Define structs that map data directly from HTML structure.
- Select elements with `#[component(selector = "...")]`.
- Extract text, attributes, or nested components.
- Support for fixed/variable-size arrays, nonempty lists, single values, optional/required attributes.
- CSS selectors are checked at compile-time (during macro resolution) and are held as static references to avoid constant re-parsing


## Example
```rust
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
<div class="item" id="cat">Meow</div>
</body>
"#;
    #[derive(Component, Debug, Clone, PartialEq)]
    struct OptionalIdChild {
        #[component]
        text: Single<String>,
        // id is optional
        #[component(map = "scraper_component::attribute::id_opt")]
        id: Single<Option<String>>,
    }

    #[derive(Component, Clone, PartialEq)]
    struct ExampleStruct {
        ///  inlines entire text from matching elements
        #[component(selector = "div.item")]
        children_simple: [String; 3],
        /// parses nested component for more granular parsing
        #[component(selector = "div.item")]
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
                    text: ["Hello".into()],
                    id: [None]
                },
                OptionalIdChild {
                    text: ["Hi".into()],
                    id: [None]
                },
                OptionalIdChild {
                    text: ["Meow".into()],
                    id: [Some("cat".into())]
                },
            ]),);
            Ok(())
        })
        .flatten()
    }
}
```

generates following code (cleaned up for readability):
```rust
pub mod three_texts_generated {
    use {
        ::scraper_component::{
            anyhow::Result,
            scraper::{ElementRef, Selector},
        },
        scraper_component::{Single, anyhow::Context},
        std::sync::LazyLock,
    };

    struct OptionalIdChild {
        text: Single<String>,
        id: Single<Option<String>>,
    }
    impl<'document> ::scraper_component::TryFromElement<'document> for OptionalIdChild {
        fn try_from_element(___element: ElementRef<'document>) -> Result<Self> {
            use ::scraper_component::anyhow::Context;
            let text = {
                static SELECTOR: LazyLock<Option<Selector>> = LazyLock::new(|| None);
                let selector = &*SELECTOR;
                let select = selector
                    .as_ref()
                    .map(|selector| Box::new(___element.select(selector)) as Box<dyn Iterator<Item = _>>)
                    .unwrap_or_else(|| Box::new(std::iter::once(___element)));
                let mapped = select.map(::scraper_component::try_from_element);
                <_ as ::scraper_component::TryCollectFrom<_>>::try_collect(mapped).with_context(|| format!("reading {}::{}", "OptionalIdChild", "text"))
            }?;

            let id = {
                static SELECTOR: LazyLock<Option<Selector>> = LazyLock::new(|| None);
                let selector = &*SELECTOR;
                let select = selector
                    .as_ref()
                    .map(|selector| Box::new(___element.select(selector)) as Box<dyn Iterator<Item = _>>)
                    .unwrap_or_else(|| Box::new(std::iter::once(___element)));
                let mapped = select.map(scraper_component::attribute::id_opt);
                <_ as ::scraper_component::TryCollectFrom<_>>::try_collect(mapped).with_context(|| format!("reading {}::{}", "OptionalIdChild", "id"))
            }?;
            Ok(Self { text, id })
        }
    }
    struct ExampleStruct {
        ///  inlines entire text from matching elements
        children_simple: [String; 3],
        /// parses nested component for more granular parsing
        children_via_struct: [OptionalIdChild; 3],
    }
    impl<'document> ::scraper_component::TryFromElement<'document> for ExampleStruct {
        fn try_from_element(___element: ElementRef<'document>) -> Result<Self> {
            let children_simple = {
                static SELECTOR: LazyLock<Option<Selector>> = LazyLock::new(|| Some(Selector::parse("div.item").expect("validated at compile time")));
                let selector = &*SELECTOR;
                let select = selector
                    .as_ref()
                    .map(|selector| Box::new(___element.select(selector)) as Box<dyn Iterator<Item = _>>)
                    .unwrap_or_else(|| Box::new(std::iter::once(___element)));
                let mapped = select.map(::scraper_component::try_from_element);
                <_ as ::scraper_component::TryCollectFrom<_>>::try_collect(mapped)
                    .with_context(|| format!("reading {}::{}", "ExampleStruct", "children_simple"))
            }?;
            let children_via_struct = {
                static SELECTOR: LazyLock<Option<Selector>> = LazyLock::new(|| Some(Selector::parse("div.item").expect("validated at compile time")));
                let selector = &*SELECTOR;
                let select = selector
                    .as_ref()
                    .map(|selector| Box::new(___element.select(selector)) as Box<dyn Iterator<Item = _>>)
                    .unwrap_or_else(|| Box::new(std::iter::once(___element)));
                let mapped = select.map(::scraper_component::try_from_element);
                <_ as ::scraper_component::TryCollectFrom<_>>::try_collect(mapped)
                    .with_context(|| format!("reading {}::{}", "ExampleStruct", "children_via_struct"))
            }?;
            Ok(Self {
                children_simple,
                children_via_struct,
            })
        }
    }
}
```
