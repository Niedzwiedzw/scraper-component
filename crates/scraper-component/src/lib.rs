#![allow(incomplete_features)]
#![feature(array_try_from_fn)]
#![feature(array_try_map)]
#![feature(adt_const_params)]
#![feature(unsized_const_params)]

pub use {anyhow, nonempty::NonEmpty, scraper, scraper_component_macros::Component};
use {
    anyhow::{Context, Result},
    itertools::Itertools,
    std::{collections::VecDeque, convert::identity, str::FromStr},
};

/// Exactly one element
pub type Single<T> = [T; 1];

pub trait TryCollectFrom<T>: Sized {
    fn try_collect<I>(from: I) -> Result<Self>
    where
        I: Iterator<Item = Result<T>>;
}

macro_rules! impl_normal_collect {
    ($ty:ty) => {
        impl<T> TryCollectFrom<T> for $ty {
            fn try_collect<I>(from: I) -> Result<Self>
            where
                I: Iterator<Item = Result<T>>,
            {
                from.process_results(|r| r.collect())
            }
        }
    };
}

#[macro_export]
macro_rules! sel {
    ($selector:expr) => {
        &::scraper_component::scraper::Selector::parse($selector).expect("parsing selector")
    };
}

impl<T> TryCollectFrom<T> for Option<T> {
    fn try_collect<I>(from: I) -> Result<Self>
    where
        I: Iterator<Item = Result<T>>,
    {
        from.process_results(|mut r| r.next())
    }
}

impl_normal_collect!(Vec<T>);
impl_normal_collect!(VecDeque<T>);

impl<T> TryCollectFrom<T> for nonempty::NonEmpty<T> {
    fn try_collect<I>(from: I) -> Result<Self>
    where
        I: Iterator<Item = Result<T>>,
    {
        from.process_results(|r| nonempty::NonEmpty::collect(r).context("expected at least one element"))
            .flatten()
    }
}

impl<const SIZE: usize, T> TryCollectFrom<T> for [T; SIZE] {
    fn try_collect<I>(mut from: I) -> Result<Self>
    where
        I: Iterator<Item = Result<T>>,
    {
        std::array::try_from_fn(|idx| {
            from.next()
                .with_context(|| format!("could not extract element at index [{idx}]"))
        })
        .and_then(|e| e.try_map(identity))
        .and_then(|output| match from.next() {
            Some(_) => Err(anyhow::anyhow!("found extra element")),
            None => Ok(output),
        })
    }
}

pub fn try_from_element<'document, T>(element: scraper::ElementRef<'document>) -> Result<T>
where
    T: TryFromElement<'document> + Sized + 'document,
{
    T::try_from_element(element)
}

pub trait TryFromElement<'document>: Sized + 'document {
    fn try_from_element(element: scraper::ElementRef<'document>) -> Result<Self>;
}

impl<'document> TryFromElement<'document> for scraper::ElementRef<'document> {
    fn try_from_element(element: scraper::ElementRef<'document>) -> Result<Self> {
        Ok(element)
    }
}

impl<'document> TryFromElement<'document> for String {
    fn try_from_element(element: scraper::ElementRef<'document>) -> Result<Self> {
        Ok(element.text().join(""))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Hardcoded<const VALUE: &'static str>;

impl<'document, const VALUE: &'static str> TryFromElement<'document> for Hardcoded<VALUE> {
    fn try_from_element(element: scraper::ElementRef<'document>) -> Result<Self> {
        let text = element.text().join("");
        if text.trim() == VALUE {
            Ok(Self)
        } else {
            Err(anyhow::anyhow!("expected {VALUE}, found {text}"))
        }
    }
}

#[derive(Debug)]
pub struct Parsed<T>(pub T);

impl<'document, T> TryFromElement<'document> for Parsed<T>
where
    T: FromStr + 'document,
    T::Err: std::error::Error + Send + Sync + 'static,
{
    fn try_from_element(element: scraper::ElementRef<'document>) -> Result<Self> {
        element
            .text()
            .join("")
            .parse::<T>()
            .with_context(|| format!("could not parse into [{}]", std::any::type_name::<Self>()))
            .map(Parsed)
    }
}

pub mod attribute;
