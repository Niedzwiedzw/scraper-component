use anyhow::Context;

pub fn syntax_highlighting(input: String) -> String {
    use syntect::{highlighting::ThemeSet, parsing::SyntaxSet, util::as_24_bit_terminal_escaped};
    let ss = SyntaxSet::load_defaults_newlines();
    let ts = ThemeSet::load_defaults();
    let syntax = ss.find_syntax_by_extension("rs").expect("no rs syntax");

    // Highlight the code
    let mut h = syntect::easy::HighlightLines::new(syntax, ts.themes.get("base16-ocean.dark").expect("no such theme"));
    input
        .lines()
        .map(|line| {
            h.highlight_line(line, &ss)
                .context("higlighting line")
                .map(|line| as_24_bit_terminal_escaped(&line[..], false))
                .unwrap_or_else(|reason| format!("{line} // {reason:?}"))
        })
        .collect::<Vec<_>>()
        .join("\n")
}

pub fn format_macro_output(tokens: &proc_macro2::TokenStream) -> String {
    syn::parse2::<syn::File>(tokens.clone())
        .map(|parsed| prettyplease::unparse(&parsed))
        .unwrap_or_else(|reason| format!("// FAILED TO FORMAT: {reason:?}\n{tokens}"))
}

pub trait ResultZipExt<T, E> {
    fn zip<U>(self, other: std::result::Result<U, E>) -> std::result::Result<(T, U), E>;
}

impl<T, E> ResultZipExt<T, E> for std::result::Result<T, E> {
    fn zip<U>(self, other: std::result::Result<U, E>) -> std::result::Result<(T, U), E> {
        match (self, other) {
            (Ok(t), Ok(u)) => Ok((t, u)),
            (Err(e), _) | (_, Err(e)) => Err(e),
        }
    }
}
