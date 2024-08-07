use alloc::borrow::ToOwned;
use alloc::string::String;
use core::fmt;
use core::{borrow::Borrow, ops::Deref, str::FromStr};

use crate::ParseXsd;

#[derive(Debug, thiserror::Error)]
#[error("invalid normalized string `{0}`")]
pub struct InvalidNormalizedStr<T = String>(pub T);

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(transparent)]
pub struct NormalizedStr(str);

impl NormalizedStr {
	pub fn new(value: &str) -> Result<&Self, InvalidNormalizedStr<&str>> {
		if Self::validate(value) {
			Ok(unsafe { Self::new_unchecked(value) })
		} else {
			Err(InvalidNormalizedStr(value))
		}
	}

	fn validate(value: &str) -> bool {
		value.chars().all(|c| !matches!(c, '\t' | '\n' | '\r'))
	}

	/// Creates a new normalized string from the input `value` without
	/// validation.
	///
	/// # Safety
	///
	/// The input `value` must be an XSD normalized string.
	pub unsafe fn new_unchecked(value: &str) -> &Self {
		core::mem::transmute(value)
	}

	pub fn as_str(&self) -> &str {
		&self.0
	}
}

impl fmt::Display for NormalizedStr {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		self.0.fmt(f)
	}
}

impl ToOwned for NormalizedStr {
	type Owned = NormalizedString;

	fn to_owned(&self) -> Self::Owned {
		NormalizedString(self.0.to_owned())
	}
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct NormalizedString(String);

impl NormalizedString {
	pub fn new(value: String) -> Result<Self, InvalidNormalizedStr> {
		if NormalizedStr::validate(&value) {
			Ok(Self(value))
		} else {
			Err(InvalidNormalizedStr(value))
		}
	}

	/// Creates a new normalized string from the input `value` without
	/// validation.
	///
	/// # Safety
	///
	/// The input `value` must be an XSD normalized string.
	pub unsafe fn new_unchecked(value: String) -> Self {
		Self(value)
	}

	pub fn as_normalized_str(&self) -> &NormalizedStr {
		unsafe { NormalizedStr::new_unchecked(self.0.as_str()) }
	}

	pub fn into_string(self) -> String {
		self.0
	}
}

impl fmt::Display for NormalizedString {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		self.0.fmt(f)
	}
}

impl Borrow<NormalizedStr> for NormalizedString {
	fn borrow(&self) -> &NormalizedStr {
		self.as_normalized_str()
	}
}

impl Deref for NormalizedString {
	type Target = NormalizedStr;

	fn deref(&self) -> &Self::Target {
		self.as_normalized_str()
	}
}

impl FromStr for NormalizedString {
	type Err = InvalidNormalizedStr<String>;

	fn from_str(s: &str) -> Result<Self, Self::Err> {
		Self::new(s.to_owned())
	}
}

impl ParseXsd for NormalizedString {
	type LexicalForm = crate::lexical::NormalizedStr;
}
