use super::{ExpandError, ExpandResult, JsonLdProcessor, Options};
use crate::context_processing::Process;
use crate::expansion::Expand;
use crate::{Context, Loader, RemoteDocument, RemoteDocumentReference};
use alloc::boxed::Box;
use core::future::Future;
use core::hash::Hash;
use core::pin::Pin;
use rdf_types::VocabularyMut;

impl<I> JsonLdProcessor<I> for RemoteDocument<I> {
	fn expand_full<'a, N>(
		&'a self,
		vocabulary: &'a mut N,
		loader: &'a impl Loader,
		mut options: Options<I>,
	) -> Pin<Box<dyn Future<Output = ExpandResult<I, N::BlankId>> + 'a>>
	where
		N: VocabularyMut<Iri = I>,
		I: Clone + Eq + Hash,
		N::BlankId: Clone + Eq + Hash,
	{
		Box::pin(async move {
			let mut active_context =
				Context::new(options.base.clone().or_else(|| self.url().cloned()));

			if let Some(expand_context) = options.expand_context.take() {
				active_context = expand_context
					.load_context_with(vocabulary, loader)
					.await
					.map_err(ExpandError::ContextLoading)?
					.into_document()
					.process_full(
						vocabulary,
						&active_context,
						loader,
						active_context.original_base_url().cloned(),
						options.context_processing_options(),
					)
					.await
					.map_err(ExpandError::ContextProcessing)?
					.into_processed()
			};

			if let Some(context_url) = self.context_url() {
				active_context = RemoteDocumentReference::Iri(context_url.clone())
					.load_context_with(vocabulary, loader)
					.await
					.map_err(ExpandError::ContextLoading)?
					.into_document()
					.process_full(
						vocabulary,
						&active_context,
						loader,
						Some(context_url.clone()),
						options.context_processing_options(),
					)
					.await
					.map_err(ExpandError::ContextProcessing)?
					.into_processed()
			}

			self.document()
				.expand_full(
					vocabulary,
					active_context,
					self.url().or(options.base.as_ref()).cloned(),
					loader,
					options.expansion_options(),
				)
				.await
				.map_err(ExpandError::Expansion)
		})
	}
}

impl<I> JsonLdProcessor<I> for RemoteDocumentReference<I, json_syntax::Value> {
	fn expand_full<'a, N>(
		&'a self,
		vocabulary: &'a mut N,
		loader: &'a impl Loader,
		options: Options<I>,
	) -> Pin<Box<dyn Future<Output = ExpandResult<I, N::BlankId>> + 'a>>
	where
		N: VocabularyMut<Iri = I>,
		I: Clone + Eq + Hash,
		N::BlankId: Clone + Eq + Hash,
	{
		Box::pin(async move {
			let doc = self.loaded_with(vocabulary, loader).await?;
			JsonLdProcessor::expand_full(doc.as_ref(), vocabulary, loader, options).await
		})
	}
}
