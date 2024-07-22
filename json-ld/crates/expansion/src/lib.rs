//! This library implements the [JSON-LD expansion algorithm](https://www.w3.org/TR/json-ld-api/#expansion-algorithms)
//! for the [`json-ld` crate](https://crates.io/crates/json-ld).
//!
//! # Usage
//!
//! The expansion algorithm is provided by the [`Expand`] trait.

#![cfg_attr(not(feature = "std"), no_std)]

extern crate alloc;

extern crate thiserror_nostd_notrait as thiserror;

use alloc::boxed::Box;
use core::future::Future;
use core::hash::Hash;
use core::pin::Pin;

use json_ld_context_processing::Context;
use json_ld_core::{Environment, ExpandedDocument, Loader, RemoteDocument};
use json_syntax::Value;
use rdf_types::{vocabulary, vocabulary::BlankIdVocabulary, BlankIdBuf, VocabularyMut};

mod array;
mod document;
mod element;
mod error;
mod expanded;
mod literal;
mod node;
mod options;
mod value;
mod warning;

pub use error::*;
pub use expanded::*;
pub use options::*;
pub use warning::*;

pub(crate) use array::*;
pub(crate) use document::filter_top_level_item;
pub(crate) use element::*;
pub(crate) use json_ld_context_processing::algorithm::expand_iri_simple as expand_iri;
pub(crate) use literal::*;
pub(crate) use node::*;
pub(crate) use value::*;

/// Result of the document expansion.
pub type ExpansionResult<T, B> = Result<ExpandedDocument<T, B>, Error>;

/// Handler for the possible warnings emitted during the expansion
/// of a JSON-LD document.
pub trait WarningHandler<N: BlankIdVocabulary>:
	json_ld_core::warning::Handler<N, Warning<N::BlankId>>
{
}

impl<N: BlankIdVocabulary, H> WarningHandler<N> for H where
	H: json_ld_core::warning::Handler<N, Warning<N::BlankId>>
{
}

/// Document expansion.
///
/// This trait provides the functions necessary to expand
/// a JSON-LD document into an [`ExpandedDocument`].
/// It is implemented by [`json_syntax::Value`] representing
/// a JSON object and [`RemoteDocument`].
///
/// # Example
///
/// ```
/// # mod json_ld { pub use json_ld_syntax as syntax; pub use json_ld_core::{RemoteDocument, ExpandedDocument, NoLoader}; pub use json_ld_expansion::Expand; };
///
/// use iref::IriBuf;
/// use rdf_types::BlankIdBuf;
/// use static_iref::iri;
/// use json_ld::{syntax::Parse, RemoteDocument, Expand};
///
/// # #[async_std::test]
/// # async fn example() {
/// // Parse the input JSON(-LD) document.
/// let (json, _) = json_ld::syntax::Value::parse_str(
///   r##"
///   {
///     "@graph": [
///       {
///         "http://example.org/vocab#a": {
///           "@graph": [
///             {
///               "http://example.org/vocab#b": "Chapter One"
///             }
///           ]
///         }
///       }
///     ]
///   }
///   "##)
/// .unwrap();
///
/// // Prepare a dummy document loader using [`json_ld::NoLoader`],
/// // since we won't need to load any remote document while expanding this one.
/// let mut loader = json_ld::NoLoader;
///
/// // The `expand` method returns an [`json_ld::ExpandedDocument`].
/// json
///     .expand(&mut loader)
///     .await
///     .unwrap();
/// # }
/// ```
pub trait Expand<Iri> {
	/// Returns the default base URL passed to the expansion algorithm
	/// and used to initialize the default empty context when calling
	/// [`Expand::expand`] or [`Expand::expand_with`].
	fn default_base_url(&self) -> Option<Iri>;

	/// Expand the document with full options.
	///
	/// The `vocabulary` is used to interpret identifiers.
	/// The `context` is used as initial context.
	/// The `base_url` is the initial base URL used to resolve relative IRI references.
	/// The given `loader` is used to load remote documents (such as contexts)
	/// imported by the input and required during expansion.
	/// The `options` are used to tweak the expansion algorithm.
	/// The `warning_handler` is called each time a warning is emitted during expansion.
	fn expand_full<'a, N, L>(
		&'a self,
		vocabulary: &'a mut N,
		context: Context<Iri, N::BlankId>,
		base_url: Option<N::Iri>,
		loader: &'a L,
		options: Options,
	) -> Pin<Box<dyn Future<Output = ExpansionResult<N::Iri, N::BlankId>> + 'a>>
	where
		N: VocabularyMut<Iri = Iri>,
		Iri: Clone + Eq + Hash,
		N::BlankId: Clone + Eq + Hash,
		L: Loader;

	/// Expand the input JSON-LD document with the given `vocabulary`
	/// to interpret identifiers.
	///
	/// The given `loader` is used to load remote documents (such as contexts)
	/// imported by the input and required during expansion.
	/// The expansion algorithm is called with an empty initial context with
	/// a base URL given by [`Expand::default_base_url`].
	fn expand_with<'a, N, L>(
		&'a self,
		vocabulary: &'a mut N,
		loader: &'a L,
	) -> Pin<Box<dyn Future<Output = ExpansionResult<Iri, N::BlankId>> + 'a>>
	where
		N: VocabularyMut<Iri = Iri>,
		Iri: 'a + Clone + Eq + Hash,
		N::BlankId: 'a + Clone + Eq + Hash,
		L: Loader,
	{
		self.expand_full(
			vocabulary,
			Context::<N::Iri, N::BlankId>::new(self.default_base_url().clone()),
			self.default_base_url().clone(),
			loader,
			Options::default(),
		)
	}

	/// Expand the input JSON-LD document.
	///
	/// The given `loader` is used to load remote documents (such as contexts)
	/// imported by the input and required during expansion.
	/// The expansion algorithm is called with an empty initial context with
	/// a base URL given by [`Expand::default_base_url`].
	fn expand<'a, L>(
		&'a self,
		loader: &'a L,
	) -> Pin<Box<dyn Future<Output = ExpansionResult<Iri, BlankIdBuf>> + 'a>>
	where
		(): VocabularyMut<Iri = Iri>,
		Iri: 'a + Clone + Eq + Hash,
		L: Loader,
	{
		self.expand_with(vocabulary::no_vocabulary_mut(), loader)
	}
}

/// Value expansion without base URL.
impl<Iri> Expand<Iri> for Value {
	fn default_base_url(&self) -> Option<Iri> {
		None
	}

	fn expand_full<'a, N, L>(
		&'a self,
		vocabulary: &'a mut N,
		context: Context<Iri, N::BlankId>,
		base_url: Option<N::Iri>,
		loader: &'a L,
		options: Options,
	) -> Pin<Box<dyn Future<Output = ExpansionResult<N::Iri, N::BlankId>> + 'a>>
	where
		N: VocabularyMut<Iri = Iri>,
		Iri: Clone + Eq + Hash,
		N::BlankId: Clone + Eq + Hash,
		L: Loader,
	{
		document::expand(
			Environment { vocabulary, loader },
			self,
			context,
			base_url,
			options,
		)
	}
}

/// Remote document expansion.
///
/// The default base URL given to the expansion algorithm is the URL of
/// the remote document.
impl<Iri: Clone> Expand<Iri> for RemoteDocument<Iri> {
	fn default_base_url(&self) -> Option<Iri> {
		self.url().cloned()
	}

	fn expand_full<'a, N, L>(
		&'a self,
		vocabulary: &'a mut N,
		context: Context<Iri, N::BlankId>,
		base_url: Option<N::Iri>,
		loader: &'a L,
		options: Options,
	) -> Pin<Box<dyn Future<Output = ExpansionResult<N::Iri, N::BlankId>> + 'a>>
	where
		N: VocabularyMut<Iri = Iri>,
		Iri: Clone + Eq + Hash,
		N::BlankId: Clone + Eq + Hash,
		L: Loader,
	{
		self.document()
			.expand_full(vocabulary, context, base_url, loader, options)
	}
}
