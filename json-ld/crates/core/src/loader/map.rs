use super::{Loader, RemoteDocument};
use crate::{LoadError, LoadingResult};
use alloc::borrow::ToOwned;
use alloc::boxed::Box;
use alloc::collections::BTreeMap;
use core::future::Future;
use core::pin::Pin;
use hashbrown::HashMap;
use iref::{Iri, IriBuf};

/// Error returned using [`HashMap`] or [`BTreeMap`] as a [`Loader`] with the
/// requested document is not found.
#[derive(Debug, thiserror::Error)]
#[error("document not found")]
pub struct EntryNotFound;

#[cfg(not(feature = "std"))]
impl crate::Convenient for EntryNotFound {}

impl Loader for HashMap<IriBuf, RemoteDocument> {
	fn load<'a>(
		&'a self,
		url: &'a Iri,
	) -> Pin<Box<dyn Future<Output = LoadingResult<IriBuf>> + 'a>> {
		Box::pin(async move {
			match self.get(url) {
				Some(document) => Ok(document.clone()),
				None => Err(LoadError::new(url.to_owned(), EntryNotFound)),
			}
		})
	}
}

impl Loader for BTreeMap<IriBuf, RemoteDocument> {
	fn load<'a>(
		&'a self,
		url: &'a Iri,
	) -> Pin<Box<dyn Future<Output = LoadingResult<IriBuf>> + 'a>> {
		Box::pin(async move {
			match self.get(url) {
				Some(document) => Ok(document.clone()),
				None => Err(LoadError::new(url.to_owned(), EntryNotFound)),
			}
		})
	}
}
