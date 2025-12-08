# scraper-component

A Rust library providing a procedural macro (`#[derive(Component)]`) and helpers for declaratively parsing HTML elements using the `scraper` crate.

## Features

- Define structs that map directly to HTML structures.
- Select elements with `#[component(selector = "...")]`.
- Extract text, attributes, or nested components.
- Support for fixed-size arrays, single values, optional/required attributes.

## Example

```rust
#[derive(Component, Debug, Clone)]
struct ExampleStruct {
    #[component(selector = "div.item")]
    children: [String; 3],

    #[component(selector = "div.item")]
    children_2: [OptionalIdChild; 3],
}

#[derive(Component, Debug, Clone)]
struct OptionalIdChild {
    #[component(map = "scraper_component::attribute::id_opt")]
    id: Single<Option<String>>,
}
