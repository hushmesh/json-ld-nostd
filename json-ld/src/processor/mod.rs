use crate::context_processing;
use crate::expansion;
use crate::syntax::ErrorCode;
use crate::{flattening::ConflictingIndexes, ExpandedDocument, Loader, ProcessingMode};
use alloc::boxed::Box;
use core::future::Future;
use core::hash::Hash;
use core::pin::Pin;
use iref::IriBuf;
use json_ld_core::rdf::RdfDirection;
use json_ld_core::{ContextLoadError, LoadError};
use json_ld_core::{Document, RemoteContextReference};
use rdf_types::{vocabulary, BlankIdBuf, VocabularyMut};

mod remote_document;

/// JSON-LD Processor options.
#[derive(Clone)]
pub struct Options<I = IriBuf> {
	/// The base IRI to use when expanding or compacting the document.
	///
	/// If set, this overrides the input document's IRI.
	pub base: Option<I>,

	/// If set to true, the JSON-LD processor replaces arrays with just one element with that element during compaction.
	///
	/// If set to false, all arrays will remain arrays even if they have just one element.
	///
	/// Defaults to `true`.
	pub compact_arrays: bool,

	/// Determines if IRIs are compacted relative to the base option or document
	/// location when compacting.
	///
	/// Defaults to `true`.
	pub compact_to_relative: bool,

	/// A context that is used to initialize the active context when expanding a document.
	pub expand_context: Option<RemoteContextReference<I>>,

	/// If set to `true`, certain algorithm processing steps where indicated are
	/// ordered lexicographically.
	///
	/// If `false`, order is not considered in processing.
	///
	/// Defaults to `false`.
	pub ordered: bool,

	/// Sets the processing mode.
	///
	/// Defaults to `ProcessingMode::JsonLd1_1`.
	pub processing_mode: ProcessingMode,

	/// Determines how value objects containing a base direction are transformed
	/// to and from RDF.
	///
	///   - If set to [`RdfDirection::I18nDatatype`], an RDF literal is
	/// generated using a datatype IRI based on <https://www.w3.org/ns/i18n#>
	/// with both the language tag (if present) and base direction encoded. When
	/// transforming from RDF, this datatype is decoded to create a value object
	/// containing `@language` (if present) and `@direction`.
	///   - If set to [`RdfDirection::CompoundLiteral`], a blank node is emitted
	/// instead of a literal, where the blank node is the subject of
	/// `rdf:value`, `rdf:direction`, and `rdf:language` (if present)
	/// properties. When transforming from RDF, this object is decoded to create
	/// a value object containing `@language` (if present) and `@direction`.
	pub rdf_direction: Option<RdfDirection>,

	/// If set to `true`, the JSON-LD processor may emit blank nodes for triple
	/// predicates, otherwise they will be omitted.
	/// See <https://www.w3.org/TR/rdf11-concepts/>.
	///
	/// The use of blank node identifiers to label properties is obsolete, and
	/// may be removed in a future version of JSON-LD, as is the support for
	/// generalized RDF Datasets and thus this option
	/// may be also be removed.
	pub produce_generalized_rdf: bool,

	/// Term expansion policy, passed to the document expansion algorithm.
	pub expansion_policy: expansion::Policy,
}

impl<I> Options<I> {
	/// Returns these options with the `ordered` flag set to `false`.
	///
	/// This means entries will not be ordered by keys before being processed.
	pub fn unordered(self) -> Self {
		Self {
			ordered: false,
			..self
		}
	}

	/// Returns these options with the `expand_context` set to the given
	/// `context`.
	pub fn with_expand_context(self, context: RemoteContextReference<I>) -> Self {
		Self {
			expand_context: Some(context),
			..self
		}
	}

	/// Builds options for the context processing algorithm from these options.
	pub fn context_processing_options(&self) -> context_processing::Options {
		context_processing::Options {
			processing_mode: self.processing_mode,
			..Default::default()
		}
	}

	/// Builds options for the expansion algorithm from these options.
	pub fn expansion_options(&self) -> expansion::Options {
		expansion::Options {
			processing_mode: self.processing_mode,
			ordered: self.ordered,
			policy: self.expansion_policy,
		}
	}
}

impl<I> Default for Options<I> {
	fn default() -> Self {
		Self {
			base: None,
			compact_arrays: true,
			compact_to_relative: true,
			expand_context: None,
			ordered: false,
			processing_mode: ProcessingMode::JsonLd1_1,
			rdf_direction: None,
			produce_generalized_rdf: false,
			expansion_policy: expansion::Policy::default(),
		}
	}
}

/// Error that can be raised by the [`JsonLdProcessor::expand`] function.
#[derive(Debug, thiserror::Error)]
pub enum ExpandError {
	/// Document expansion failed.
	#[error("Expansion failed: {0}")]
	Expansion(expansion::Error),

	/// Context processing failed.
	#[error("Context processing failed: {0}")]
	ContextProcessing(context_processing::Error),

	/// Remote document loading failed with the given precise error.
	#[error(transparent)]
	Loading(#[from] LoadError),

	#[error(transparent)]
	ContextLoading(ContextLoadError),
}

impl ExpandError {
	/// Returns the code of this error.
	pub fn code(&self) -> ErrorCode {
		match self {
			Self::Expansion(e) => e.code(),
			Self::ContextProcessing(e) => e.code(),
			Self::Loading(_) => ErrorCode::LoadingDocumentFailed,
			Self::ContextLoading(_) => ErrorCode::LoadingRemoteContextFailed,
		}
	}
}

/// Result returned by the [`JsonLdProcessor::expand`] function.
pub type ExpandResult<I, B> = Result<ExpandedDocument<I, B>, ExpandError>;

/// Result returned by the [`JsonLdProcessor::into_document`] function.
pub type IntoDocumentResult<I, B> = Result<Document<I, B>, ExpandError>;

/// Error that can be raised by the [`JsonLdProcessor::compact`] function.
#[derive(Debug, thiserror::Error)]
pub enum CompactError {
	/// Document expansion failed.
	#[error("Expansion failed: {0}")]
	Expand(ExpandError),

	/// Context processing failed.
	#[error("Context processing failed: {0}")]
	ContextProcessing(context_processing::Error),

	/// Remote document loading failed.
	#[error(transparent)]
	Loading(#[from] LoadError),

	#[error(transparent)]
	ContextLoading(ContextLoadError),
}

/// Error that can be raised by the [`JsonLdProcessor::flatten`] function.
#[derive(Debug, thiserror::Error)]
pub enum FlattenError<I, B> {
	#[error("Expansion failed: {0}")]
	Expand(ExpandError),

	#[error("Conflicting indexes: {0}")]
	ConflictingIndexes(ConflictingIndexes<I, B>),

	#[error(transparent)]
	Loading(#[from] LoadError),

	#[error(transparent)]
	ContextLoading(ContextLoadError),
}

impl<I, B> FlattenError<I, B> {
	/// Returns the code of this error.
	pub fn code(&self) -> ErrorCode {
		match self {
			Self::Expand(e) => e.code(),
			Self::ConflictingIndexes(_) => ErrorCode::ConflictingIndexes,
			Self::Loading(_) => ErrorCode::LoadingDocumentFailed,
			Self::ContextLoading(_) => ErrorCode::LoadingRemoteContextFailed,
		}
	}
}

/// Application Programming Interface.
///
/// The `JsonLdProcessor` interface is the high-level programming structure that
/// developers use to access the JSON-LD transformation methods.
///
/// It is notably implemented for the [`RemoteDocument<I, M, json_syntax::Value<M>>`](crate::RemoteDocument)
/// and [`RemoteDocumentReference<I, M, json_syntax::Value<M>>`] types.
///
/// # Methods naming
///
/// Each processing function is declined in four variants depending on your
/// needs, with the following suffix convention:
///
///   - `_full`: function with all the possible options. This is the only way
///     to specify a custom warning handler.
///   - `_with`: allows passing a custom [`Vocabulary`].
///   - `_using`: allows passing custom [`Options`].
///   - `_with_using`: allows passing both a custom [`Vocabulary`] and
///     custom [`Options`].
///   - no suffix: minimum parameters. No custom vocabulary: [`IriBuf`] and
///     [`BlankIdBuf`] must be used as IRI and blank node id respectively.
///
/// [`IriBuf`]: https://docs.rs/iref/latest/iref/struct.IriBuf.html
/// [`BlankIdBuf`]: rdf_types::BlankIdBuf
/// [`Vocabulary`]: rdf_types::Vocabulary
///
/// # Example
///
/// ```
/// use static_iref::iri;
/// use json_ld::{JsonLdProcessor, RemoteDocumentReference};
///
/// # #[async_std::main]
/// # async fn main() {
/// let input = RemoteDocumentReference::iri(iri!("https://example.com/sample.jsonld").to_owned());
///
/// // Use `FsLoader` to redirect any URL starting with `https://example.com/` to
/// // the local `example` directory. No HTTP query.
/// let mut loader = json_ld::FsLoader::default();
/// loader.mount(iri!("https://example.com/").to_owned(), "examples");
///
/// let expanded = input.expand(&loader)
///   .await
///   .expect("expansion failed");
/// # }
/// ```
pub trait JsonLdProcessor<Iri>: Sized {
	/// Expand the document with the given `vocabulary` and `loader`, using
	/// the given `options` and warning handler.
	///
	/// On success, the result is an [`ExpandedDocument`].
	///
	/// # Example
	///
	/// ```
	/// use static_iref::iri;
	/// use json_ld::{JsonLdProcessor, Options, RemoteDocumentReference, warning};
	/// use rdf_types::vocabulary::{IriVocabularyMut, IndexVocabulary};
	/// # #[async_std::main]
	/// # async fn main() {
	/// // Creates the vocabulary that will map each `rdf_types::vocabulary::Index`
	/// // to an actual `IriBuf`.
	/// let mut vocabulary: IndexVocabulary = IndexVocabulary::new();
	///
	/// let iri_index = vocabulary.insert(iri!("https://example.com/sample.jsonld"));
	/// let input = RemoteDocumentReference::iri(iri_index);
	///
	/// // Use `FsLoader` to redirect any URL starting with `https://example.com/` to
	/// // the local `example` directory. No HTTP query.
	/// let mut loader = json_ld::FsLoader::default();
	/// loader.mount(iri!("https://example.com/").to_owned(), "examples");
	///
	/// let expanded = input
	///   .expand_full(
	///     &mut vocabulary,
	///     &loader,
	///     Options::default(),
	///     warning::PrintWith
	///   )
	///   .await
	///   .expect("expansion failed");
	/// # }
	/// ```
	fn expand_full<'a, N>(
		&'a self,
		vocabulary: &'a mut N,
		loader: &'a impl Loader,
		options: Options<Iri>,
	) -> Pin<Box<dyn Future<Output = ExpandResult<Iri, N::BlankId>> + 'a>>
	where
		N: VocabularyMut<Iri = Iri>,
		Iri: Clone + Eq + Hash,
		N::BlankId: Clone + Eq + Hash;

	/// Expand the document with the given `vocabulary` and `loader`, using
	/// the given `options`.
	///
	/// Warnings are ignored.
	/// On success, the result is an [`ExpandedDocument`].
	///
	/// # Example
	///
	/// ```
	/// use static_iref::iri;
	/// use json_ld::{JsonLdProcessor, Options, RemoteDocumentReference, warning};
	/// use rdf_types::vocabulary::{IriVocabularyMut, IndexVocabulary};
	/// # #[async_std::main]
	/// # async fn main() {
	/// // Creates the vocabulary that will map each `rdf_types::vocabulary::Index`
	/// // to an actual `IriBuf`.
	/// let mut vocabulary: IndexVocabulary = IndexVocabulary::new();
	///
	/// let iri_index = vocabulary.insert(iri!("https://example.com/sample.jsonld"));
	/// let input = RemoteDocumentReference::iri(iri_index);
	///
	/// // Use `FsLoader` to redirect any URL starting with `https://example.com/` to
	/// // the local `example` directory. No HTTP query.
	/// let mut loader = json_ld::FsLoader::default();
	/// loader.mount(iri!("https://example.com/").to_owned(), "examples");
	///
	/// let expanded = input
	///   .expand_with_using(
	///     &mut vocabulary,
	///     &loader,
	///     Options::default()
	///   )
	///   .await
	///   .expect("expansion failed");
	/// # }
	/// ```
	fn expand_with_using<'a, N>(
		&'a self,
		vocabulary: &'a mut N,
		loader: &'a impl Loader,
		options: Options<Iri>,
	) -> Pin<Box<dyn Future<Output = ExpandResult<Iri, N::BlankId>> + 'a>>
	where
		N: VocabularyMut<Iri = Iri>,
		Iri: Clone + Eq + Hash,
		N::BlankId: 'a + Clone + Eq + Hash,
	{
		self.expand_full(vocabulary, loader, options)
	}

	/// Expand the document with the given `vocabulary` and `loader`.
	///
	/// Default options are used.
	/// Warnings are ignored.
	/// On success, the result is an [`ExpandedDocument`].
	///
	/// # Example
	///
	/// ```
	/// use static_iref::iri;
	/// use json_ld::{JsonLdProcessor, Options, RemoteDocumentReference, warning};
	/// use rdf_types::vocabulary::{IriVocabularyMut, IndexVocabulary};
	/// # #[async_std::main]
	/// # async fn main() {
	/// // Creates the vocabulary that will map each `rdf_types::vocabulary::Index`
	/// // to an actual `IriBuf`.
	/// let mut vocabulary: IndexVocabulary = IndexVocabulary::new();
	///
	/// let iri_index = vocabulary.insert(iri!("https://example.com/sample.jsonld"));
	/// let input = RemoteDocumentReference::iri(iri_index);
	///
	/// // Use `FsLoader` to redirect any URL starting with `https://example.com/` to
	/// // the local `example` directory. No HTTP query.
	/// let mut loader = json_ld::FsLoader::default();
	/// loader.mount(iri!("https://example.com/").to_owned(), "examples");
	///
	/// let expanded = input
	///   .expand_with(
	///     &mut vocabulary,
	///     &loader
	///   )
	///   .await
	///   .expect("expansion failed");
	/// # }
	/// ```
	fn expand_with<'a, N>(
		&'a self,
		vocabulary: &'a mut N,
		loader: &'a impl Loader,
	) -> Pin<Box<dyn Future<Output = ExpandResult<Iri, N::BlankId>> + 'a>>
	where
		N: VocabularyMut<Iri = Iri>,
		Iri: Clone + Eq + Hash,
		N::BlankId: 'a + Clone + Eq + Hash,
	{
		self.expand_with_using(vocabulary, loader, Options::default())
	}

	/// Expand the document with the given `loader` using the given `options`.
	///
	/// Warnings are ignored.
	/// On success, the result is an [`ExpandedDocument`].
	///
	/// # Example
	///
	/// ```
	/// use static_iref::iri;
	/// use json_ld::{JsonLdProcessor, Options, RemoteDocumentReference, warning};
	///
	/// # #[async_std::main]
	/// # async fn main() {
	/// let iri = iri!("https://example.com/sample.jsonld").to_owned();
	/// let input = RemoteDocumentReference::iri(iri);
	///
	/// // Use `FsLoader` to redirect any URL starting with `https://example.com/` to
	/// // the local `example` directory. No HTTP query.
	/// let mut loader = json_ld::FsLoader::default();
	/// loader.mount(iri!("https://example.com/").to_owned(), "examples");
	///
	/// let expanded = input
	///   .expand_using(
	///     &loader,
	///     Options::default()
	///   )
	///   .await
	///   .expect("expansion failed");
	/// # }
	/// ```
	fn expand_using<'a>(
		&'a self,
		loader: &'a impl Loader,
		options: Options<Iri>,
	) -> Pin<Box<dyn Future<Output = ExpandResult<Iri, BlankIdBuf>> + 'a>>
	where
		(): VocabularyMut<Iri = Iri>,
		Iri: Clone + Eq + Hash,
	{
		self.expand_with_using(vocabulary::no_vocabulary_mut(), loader, options)
	}

	/// Expand the document with the given `loader`.
	///
	/// Default options are used.
	/// Warnings are ignored.
	/// On success, the result is an [`ExpandedDocument`].
	///
	/// # Example
	///
	/// ```
	/// use static_iref::iri;
	/// use json_ld::{JsonLdProcessor, Options, RemoteDocumentReference, warning};
	///
	/// # #[async_std::main]
	/// # async fn main() {
	/// let iri = iri!("https://example.com/sample.jsonld").to_owned();
	/// let input = RemoteDocumentReference::iri(iri);
	///
	/// // Use `FsLoader` to redirect any URL starting with `https://example.com/` to
	/// // the local `example` directory. No HTTP query.
	/// let mut loader = json_ld::FsLoader::default();
	/// loader.mount(iri!("https://example.com/").to_owned(), "examples");
	///
	/// let expanded = input
	///   .expand(&loader)
	///   .await
	///   .expect("expansion failed");
	/// # }
	/// ```
	fn expand<'a>(
		&'a self,
		loader: &'a impl Loader,
	) -> Pin<Box<dyn Future<Output = ExpandResult<Iri, BlankIdBuf>> + 'a>>
	where
		(): VocabularyMut<Iri = Iri>,
		Iri: Clone + Eq + Hash,
	{
		self.expand_with(vocabulary::no_vocabulary_mut(), loader)
	}
}
