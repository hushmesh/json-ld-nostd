use super::expand_element;
use crate::{ActiveProperty, Error, Loader, Options};
use core::future::Future;
use core::hash::Hash;
use core::pin::Pin;
use json_ld_core::{Context, Environment, ExpandedDocument, IndexedObject, Object};
use json_syntax::Value;
use rdf_types::VocabularyMut;

/// Expand the given JSON-LD document.
///
/// Note that you probably do not want to use this function directly,
/// but instead use the [`Document::expand`](crate::Document::expand) method on
/// a `Value` instance.
pub(crate) fn expand<'a, N, L>(
	env: Environment<'a, N, L>,
	document: &'a Value,
	active_context: Context<N::Iri, N::BlankId>,
	base_url: Option<N::Iri>,
	options: Options,
) -> Pin<Box<dyn Future<Output = Result<ExpandedDocument<N::Iri, N::BlankId>, Error>> + 'a>>
where
	N: VocabularyMut,
	N::Iri: Clone + Eq + Hash,
	N::BlankId: Clone + Eq + Hash,
	L: Loader,
{
	Box::pin(async move {
		let expanded = expand_element(
			env,
			&active_context,
			ActiveProperty::None,
			document,
			base_url,
			options,
			false,
		)
		.await?;
		if expanded.len() == 1 {
			let obj = expanded.into_iter().next().unwrap();
			match obj.into_unnamed_graph() {
				Ok(graph) => Ok(ExpandedDocument::from(graph)),
				Err(obj) => {
					let mut result = ExpandedDocument::new();
					if filter_top_level_item(&obj) {
						result.insert(obj);
					}
					Ok(result)
				}
			}
		} else {
			Ok(expanded.into_iter().filter(filter_top_level_item).collect())
		}
	})
}

pub(crate) fn filter_top_level_item<T, B>(item: &IndexedObject<T, B>) -> bool {
	// Remove dangling values.
	!matches!(item.inner(), Object::Value(_))
}
