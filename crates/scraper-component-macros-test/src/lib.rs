#[allow(dead_code)]
#[cfg(test)]
mod tests {
    pub mod impl_struct {
        use scraper_component::{Component, Single, scraper::ElementRef};

        #[derive(Component)]
        struct ExampleStruct<'document> {
            #[component(selector = "a")]
            child: Single<ElementRef<'document>>,
        }
    }
}
