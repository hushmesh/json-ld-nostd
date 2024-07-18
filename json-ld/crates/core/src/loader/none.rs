use super::Loader;
use crate::{LoadError, LoadingResult};
use alloc::borrow::ToOwned;
use alloc::boxed::Box;
use core::future::Future;
use core::pin::Pin;
use iref::Iri;

/// Dummy loader.
///
/// A dummy loader that does not load anything.
/// Can be useful when you know that you will never need to load remote resource.
///
/// Raises an `LoadingDocumentFailed` at every attempt to load a resource.
#[derive(Debug, Default)]
pub struct NoLoader;

#[derive(Debug, thiserror::Error)]
#[error("no loader")]
pub struct CannotLoad;

impl Loader for NoLoader {
	fn load<'a>(&'a self, url: &'a Iri) -> Pin<Box<dyn Future<Output = LoadingResult> + 'a>> {
		Box::pin(async move { Err(LoadError::new(url.to_owned(), CannotLoad)) })
	}
}

#[cfg(not(feature = "std"))]
impl crate::Convenient for CannotLoad {}
