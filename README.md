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
```

generates following code (cleaned up for readability):
```rust
pub mod three_texts_generated {
    struct OptionalIdChild {
        text: String,
        id: Option<String>,
    }
    impl<'document> ::scraper_component::TryFromElement<'document> for OptionalIdChild {
        fn try_from_element(
            ___element: ::scraper_component::scraper::ElementRef<'document>,
        ) -> ::scraper_component::anyhow::Result<Self> {
            let text = {
                use ::scraper_component::{
                    anyhow::{Result, Context, anyhow},
                    scraper::Selector,
                };
                static SELECTOR: std::sync::LazyLock<
                    Option<::scraper_component::scraper::Selector>,
                > = std::sync::LazyLock::new(|| None);
                let selector = &*SELECTOR;
                let select = selector
                    .as_ref()
                    .map(|selector| {
                        (Box::new(___element.select(selector))
                            as Box<dyn Iterator<Item = _>>)
                    })
                    .unwrap_or_else(|| Box::new(std::iter::once(___element)));
                let mapped = select.map(::scraper_component::try_from_element);
                <[String; 1] as ::scraper_component::TryCollectFrom<_>>::try_collect(mapped)
                    .with_context(|| {
                        format!(
                            "reading {}::{} (selector: {}) from:\n{}", "OptionalIdChild",
                            "text", "<no-selector>", ___element.html()
                        )
                    })
                    .map(|[v]| v)
            }?;
            let id = {
                use ::scraper_component::{
                    anyhow::{Result, Context, anyhow},
                    scraper::Selector,
                };
                static SELECTOR: std::sync::LazyLock<
                    Option<::scraper_component::scraper::Selector>,
                > = std::sync::LazyLock::new(|| None);
                let selector = &*SELECTOR;
                let select = selector
                    .as_ref()
                    .map(|selector| {
                        (Box::new(___element.select(selector))
                            as Box<dyn Iterator<Item = _>>)
                    })
                    .unwrap_or_else(|| Box::new(std::iter::once(___element)));
                let mapped = select.map(scraper_component::attribute::id_opt);
                <[Option<
                    String,
                >; 1] as ::scraper_component::TryCollectFrom<_>>::try_collect(mapped)
                    .with_context(|| {
                        format!(
                            "reading {}::{} (selector: {}) from:\n{}", "OptionalIdChild",
                            "id", "<no-selector>", ___element.html()
                        )
                    })
                    .map(|[v]| v)
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
        fn try_from_element(
            ___element: ::scraper_component::scraper::ElementRef<'document>,
        ) -> ::scraper_component::anyhow::Result<Self> {
            let children_simple = {
                use ::scraper_component::{
                    anyhow::{Result, Context, anyhow},
                    scraper::Selector,
                };
                static SELECTOR: std::sync::LazyLock<
                    Option<::scraper_component::scraper::Selector>,
                > = std::sync::LazyLock::new(|| Some(
                    ::scraper_component::scraper::Selector::parse("div.item")
                        .expect("validated at compile time"),
                ));
                let selector = &*SELECTOR;
                let select = selector
                    .as_ref()
                    .map(|selector| {
                        (Box::new(___element.select(selector))
                            as Box<dyn Iterator<Item = _>>)
                    })
                    .unwrap_or_else(|| Box::new(std::iter::once(___element)));
                let mapped = select.map(::scraper_component::try_from_element);
                <[String; 3] as ::scraper_component::TryCollectFrom<_>>::try_collect(mapped)
                    .with_context(|| {
                        format!(
                            "reading {}::{} (selector: {}) from:\n{}", "ExampleStruct",
                            "children_simple", "div.item", ___element.html()
                        )
                    })
            }?;
            let children_via_struct = {
                use ::scraper_component::{
                    anyhow::{Result, Context, anyhow},
                    scraper::Selector,
                };
                static SELECTOR: std::sync::LazyLock<
                    Option<::scraper_component::scraper::Selector>,
                > = std::sync::LazyLock::new(|| Some(
                    ::scraper_component::scraper::Selector::parse("div.item")
                        .expect("validated at compile time"),
                ));
                let selector = &*SELECTOR;
                let select = selector
                    .as_ref()
                    .map(|selector| {
                        (Box::new(___element.select(selector))
                            as Box<dyn Iterator<Item = _>>)
                    })
                    .unwrap_or_else(|| Box::new(std::iter::once(___element)));
                let mapped = select.map(::scraper_component::try_from_element);
                <[OptionalIdChild; 3] as ::scraper_component::TryCollectFrom<
                    _,
                >>::try_collect(mapped)
                    .with_context(|| {
                        format!(
                            "reading {}::{} (selector: {}) from:\n{}", "ExampleStruct",
                            "children_via_struct", "div.item", ___element.html()
                        )
                    })
            }?;
            Ok(Self {
                children_simple,
                children_via_struct,
            })
        }
    }
}
```
