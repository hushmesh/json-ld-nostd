use super::lexical_form;
use alloc::borrow::ToOwned;
use alloc::string::String;
use alloc::vec::Vec;
use alloc::vec;
use core::borrow::Borrow;
use core::cmp::Ordering;
use core::fmt;
use core::hash::{Hash, Hasher};

lexical_form! {
	/// Boolean.
	///
	/// See: <https://www.w3.org/TR/xmlschema-2/#boolean>
	ty: Boolean,

	/// Owned boolean.
	///
	/// See: <https://www.w3.org/TR/xmlschema-2/#boolean>
	buffer: BooleanBuf,

	/// Creates a new boolean from a string.
	///
	/// If the input string is ot a [valid XSD boolean](https://www.w3.org/TR/xmlschema-2/#boolean),
	/// an [`InvalidBoolean`] error is returned.
	new,

	/// Creates a new boolean from a string without checking it.
	///
	/// # Safety
	///
	/// The input string must be a [valid XSD boolean](https://www.w3.org/TR/xmlschema-2/#boolean).
	new_unchecked,

	value: crate::Boolean,
	error: InvalidBoolean,
	as_ref: as_boolean,
	parent_forms: {}
}

impl Boolean {
	pub fn value(&self) -> crate::Boolean {
		crate::Boolean(matches!(&self.0, b"true" | b"1"))
	}
}

impl PartialEq for Boolean {
	fn eq(&self, other: &Self) -> bool {
		self.value() == other.value()
	}
}

impl Eq for Boolean {}

impl Hash for Boolean {
	fn hash<H: Hasher>(&self, state: &mut H) {
		self.value().hash(state)
	}
}

impl Ord for Boolean {
	fn cmp(&self, other: &Self) -> Ordering {
		self.value().cmp(&other.value())
	}
}

impl PartialOrd for Boolean {
	fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
		Some(self.cmp(other))
	}
}

impl From<bool> for BooleanBuf {
	fn from(b: bool) -> Self {
		if b {
			unsafe { BooleanBuf::new_unchecked(vec![b't', b'r', b'u', b'e']) }
		} else {
			unsafe { BooleanBuf::new_unchecked(vec![b'f', b'a', b'l', b's', b'e']) }
		}
	}
}

impl<'a> From<&'a Boolean> for crate::Boolean {
	fn from(b: &'a Boolean) -> crate::Boolean {
		b.value()
	}
}

impl From<BooleanBuf> for crate::Boolean {
	fn from(b: BooleanBuf) -> crate::Boolean {
		b.value()
	}
}

impl<'a> From<&'a Boolean> for bool {
	fn from(b: &'a Boolean) -> bool {
		b.value().into()
	}
}

impl From<BooleanBuf> for bool {
	fn from(b: BooleanBuf) -> bool {
		b.value().into()
	}
}

fn check_bytes(s: &[u8]) -> bool {
	matches!(s, b"true" | b"false" | b"0" | b"1")
}
