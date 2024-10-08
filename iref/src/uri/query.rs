use alloc::string::String;
use alloc::string::ToString;
use alloc::vec::Vec;

use pct_str::{PctStr, PctString};
use core::{
	cmp,
	hash::{self, Hash},
	ops,
};

use static_regular_grammar::RegularGrammar;

use crate::common::QueryImpl;

#[derive(RegularGrammar)]
#[grammar(
	file = "src/uri/grammar.abnf",
	entry_point = "query",
	name = "URI query",
	ascii,
	no_deref,
	cache = "automata/uri/query.aut.cbor"
)]
#[grammar(sized(QueryBuf, derive(Debug, Display, PartialEq, Eq, PartialOrd, Ord, Hash)))]
#[cfg_attr(feature = "serde", grammar(serde))]
#[cfg_attr(feature = "ignore-grammars", grammar(disable))]
pub struct Query([u8]);

impl QueryImpl for Query {
	unsafe fn new_unchecked(bytes: &[u8]) -> &Self {
		Self::new_unchecked(bytes)
	}

	fn as_bytes(&self) -> &[u8] {
		&self.0
	}
}

impl Query {
	/// Returns the query as a percent-encoded string slice.
	#[inline]
	pub fn as_pct_str(&self) -> &PctStr {
		unsafe { PctStr::new_unchecked(self.as_str()) }
	}
}

impl ops::Deref for Query {
	type Target = PctStr;

	fn deref(&self) -> &Self::Target {
		self.as_pct_str()
	}
}

impl cmp::PartialEq for Query {
	#[inline]
	fn eq(&self, other: &Query) -> bool {
		self.as_pct_str() == other.as_pct_str()
	}
}

impl Eq for Query {}

impl<'a> PartialEq<&'a str> for Query {
	#[inline]
	fn eq(&self, other: &&'a str) -> bool {
		self.as_str() == *other
	}
}

impl PartialOrd for Query {
	#[inline]
	fn partial_cmp(&self, other: &Query) -> Option<cmp::Ordering> {
		Some(self.cmp(other))
	}
}

impl Ord for Query {
	#[inline]
	fn cmp(&self, other: &Query) -> cmp::Ordering {
		self.as_pct_str().cmp(other.as_pct_str())
	}
}

impl Hash for Query {
	#[inline]
	fn hash<H: hash::Hasher>(&self, hasher: &mut H) {
		self.as_pct_str().hash(hasher)
	}
}

impl QueryBuf {
	pub fn into_pct_string(self) -> PctString {
		unsafe { PctString::new_unchecked(self.0) }
	}
}
