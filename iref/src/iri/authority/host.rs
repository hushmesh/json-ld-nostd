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

use crate::common::authority::HostImpl;

/// IRI authority host.
#[derive(RegularGrammar)]
#[grammar(
	file = "src/iri/grammar.abnf",
	entry_point = "ihost",
	name = "IRI host",
	no_deref,
	cache = "automata/iri/host.aut.cbor"
)]
#[grammar(sized(HostBuf, derive(Debug, Display, PartialEq, Eq, PartialOrd, Ord, Hash)))]
#[cfg_attr(feature = "serde", grammar(serde))]
#[cfg_attr(feature = "ignore-grammars", grammar(disable))]
pub struct Host(str);

impl HostImpl for Host {
	unsafe fn new_unchecked(bytes: &[u8]) -> &Self {
		Self::new_unchecked(core::str::from_utf8_unchecked(bytes))
	}

	fn as_bytes(&self) -> &[u8] {
		self.0.as_bytes()
	}
}

impl Host {
	/// Returns the host as a percent-encoded string slice.
	#[inline]
	pub fn as_pct_str(&self) -> &PctStr {
		unsafe { PctStr::new_unchecked(self.as_str()) }
	}
}

impl ops::Deref for Host {
	type Target = PctStr;

	fn deref(&self) -> &Self::Target {
		self.as_pct_str()
	}
}

impl cmp::PartialEq for Host {
	#[inline]
	fn eq(&self, other: &Host) -> bool {
		self.as_pct_str() == other.as_pct_str()
	}
}

impl Eq for Host {}

impl<'a> PartialEq<&'a str> for Host {
	#[inline]
	fn eq(&self, other: &&'a str) -> bool {
		self.as_str() == *other
	}
}

impl PartialOrd for Host {
	#[inline]
	fn partial_cmp(&self, other: &Host) -> Option<cmp::Ordering> {
		Some(self.cmp(other))
	}
}

impl Ord for Host {
	#[inline]
	fn cmp(&self, other: &Host) -> cmp::Ordering {
		self.as_pct_str().cmp(other.as_pct_str())
	}
}

impl Hash for Host {
	#[inline]
	fn hash<H: hash::Hasher>(&self, hasher: &mut H) {
		self.as_pct_str().hash(hasher)
	}
}

impl HostBuf {
	pub fn into_pct_string(self) -> PctString {
		unsafe { PctString::new_unchecked(self.0) }
	}
}
