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

use crate::common::UserInofImpl;

#[derive(RegularGrammar)]
#[grammar(
	file = "src/uri/grammar.abnf",
	entry_point = "userinfo",
	name = "URI user info",
	ascii,
	no_deref,
	cache = "automata/uri/userinfo.aut.cbor"
)]
#[grammar(sized(
	UserInfoBuf,
	derive(Debug, Display, PartialEq, Eq, PartialOrd, Ord, Hash)
))]
#[cfg_attr(feature = "serde", grammar(serde))]
#[cfg_attr(feature = "ignore-grammars", grammar(disable))]
pub struct UserInfo([u8]);

impl UserInofImpl for UserInfo {
	unsafe fn new_unchecked(bytes: &[u8]) -> &Self {
		Self::new_unchecked(bytes)
	}

	fn as_bytes(&self) -> &[u8] {
		&self.0
	}
}

impl UserInfo {
	/// Returns the host as a percent-encoded string slice.
	#[inline]
	pub fn as_pct_str(&self) -> &PctStr {
		unsafe { PctStr::new_unchecked(self.as_str()) }
	}
}

impl ops::Deref for UserInfo {
	type Target = PctStr;

	fn deref(&self) -> &Self::Target {
		self.as_pct_str()
	}
}

impl cmp::PartialEq for UserInfo {
	#[inline]
	fn eq(&self, other: &UserInfo) -> bool {
		self.as_pct_str() == other.as_pct_str()
	}
}

impl Eq for UserInfo {}

impl<'a> PartialEq<&'a str> for UserInfo {
	#[inline]
	fn eq(&self, other: &&'a str) -> bool {
		self.as_str() == *other
	}
}

impl PartialOrd for UserInfo {
	#[inline]
	fn partial_cmp(&self, other: &UserInfo) -> Option<cmp::Ordering> {
		Some(self.cmp(other))
	}
}

impl Ord for UserInfo {
	#[inline]
	fn cmp(&self, other: &UserInfo) -> cmp::Ordering {
		self.as_pct_str().cmp(other.as_pct_str())
	}
}

impl Hash for UserInfo {
	#[inline]
	fn hash<H: hash::Hasher>(&self, hasher: &mut H) {
		self.as_pct_str().hash(hasher)
	}
}

impl UserInfoBuf {
	pub fn into_pct_string(self) -> PctString {
		unsafe { PctString::new_unchecked(self.0) }
	}
}
