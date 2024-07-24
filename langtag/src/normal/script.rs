use alloc::string::String;
use alloc::string::ToString;
use alloc::vec::Vec;
use core::hash::Hash;

use static_regular_grammar::RegularGrammar;

use crate::utils::{self, str_eq};

/// Script subtag.
///
/// Script subtags are used to indicate the script or writing system
/// variations that distinguish the written forms of a language or its
/// dialects.
#[derive(RegularGrammar)]
#[grammar(file = "src/grammar.abnf", entry_point = "script")]
#[grammar(sized(
	ScriptBuf,
	derive(Debug, Display, PartialEq, Eq, PartialOrd, Ord, Hash)
))]
#[cfg_attr(feature = "serde", grammar(serde))]
pub struct Script(str);

impl PartialEq for Script {
	fn eq(&self, other: &Self) -> bool {
		utils::case_insensitive_eq(self.as_bytes(), other.as_bytes())
	}
}

impl Eq for Script {}

str_eq!(Script);
str_eq!(ScriptBuf);

impl PartialOrd for Script {
	fn partial_cmp(&self, other: &Self) -> Option<core::cmp::Ordering> {
		Some(self.cmp(other))
	}
}

impl Ord for Script {
	fn cmp(&self, other: &Self) -> core::cmp::Ordering {
		utils::case_insensitive_cmp(self.as_bytes(), other.as_bytes())
	}
}

impl Hash for Script {
	fn hash<H: core::hash::Hasher>(&self, state: &mut H) {
		utils::case_insensitive_hash(self.as_bytes(), state)
	}
}
