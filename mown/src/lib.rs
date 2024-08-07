//! This crate provides two simple wrappers
//! [`Mown`](crate::Mown)
//! and
//! [`MownMut`](crate::MownMut)
//! for values that can be either owned or borrowed.
//! The type `Mown` is an simple `enum` type with two constructors:
//!
//! ```rust
//! # use std::borrow::Borrow;
//! pub trait Borrowed {
//!   type Owned: Borrow<Self>;
//! }
//!
//! pub enum Mown<'a, T: Borrowed> {
//!   Owned(T::Owned),
//!   Borrowed(&'a T)
//! }
//! ```
//!
//! The mutable version `MownMut` follows the same definition with a mutable
//! reference.
//! This is very similar to the standard
//! [`Cow`](https://doc.rust-lang.org/std/borrow/enum.Cow.html)
//! type, except that it is not possible to transform a borrowed value into an owned
//! one.
//! This is also slightly different from the similar crate
//! [`boow`](https://crates.io/crates/boow)
//! since the [`ToOwned`] trait allow for the use of `Mown` with unsized types
//! (for instance `Mown<str>`) and with mutable references.
//!
//! ## Basic Usage
//!
//! One basic use case for the `Mown` type is the situation where one wants to
//! reuse some input borrowed value under some condition, or then use a custom
//! owned value.
//!
//! ```rust
//! use mown::Mown;
//!
//! fn function(input_value: &String) -> Mown<String> {
//!   # let condition = true;
//!   if condition {
//!     Mown::Borrowed(input_value)
//!   } else {
//!     let custom_value: String = "foo_".to_string() + input_value + "_bar";
//!     Mown::Owned(custom_value)
//!   }
//! }
//! ```
//!
//! One can also wrap unsized types for which the provided [`ToOwned`]
//! trait has been implemented.
//! This is the case for the unsized `str` type with the sized owned type `String`.
//!
//! ```rust
//! use mown::Mown;
//!
//! fn function(input_value: &str) -> Mown<str> {
//!   # let condition = true;
//!   if condition {
//!     Mown::Borrowed(input_value)
//!   } else {
//!     let custom_value: String = "foo_".to_string() + input_value + "_bar";
//!     Mown::Owned(custom_value)
//!   }
//! }
//! ```
#![cfg_attr(not(feature = "std"), no_std)]

extern crate alloc;

use alloc::borrow::ToOwned;
use alloc::string::String;
use alloc::vec::Vec;
use core::borrow::{Borrow, BorrowMut};
use core::cmp::{Ord, Ordering, PartialOrd};
use core::fmt::{self, Debug, Display, Formatter};
use core::hash::{Hash, Hasher};
use core::ops::{Deref, DerefMut};

/// Types that are borrowed.
pub trait Borrowed {
	type Owned: Borrow<Self>;
}

impl<T: Sized> Borrowed for T {
	type Owned = T;
}

impl Borrowed for str {
	type Owned = String;
}

impl<T> Borrowed for [T] {
	type Owned = Vec<T>;
}

/// Container for borrowed or owned value.
pub enum Mown<'a, T: ?Sized + Borrowed> {
	/// Owned value.
	Owned(T::Owned),

	/// Borrowed value.
	Borrowed(&'a T),
}

impl<'a, T: ?Sized + Borrowed> Mown<'a, T> {
	/// Checks if the value is owned.
	pub fn is_owned(&self) -> bool {
		match self {
			Mown::Owned(_) => true,
			Mown::Borrowed(_) => false,
		}
	}

	/// Checks if the value is borrowed.
	pub fn is_borrowed(&self) -> bool {
		match self {
			Mown::Owned(_) => false,
			Mown::Borrowed(_) => true,
		}
	}

	/// Returns the owned value as a mutable reference, if any.
	///
	/// If the value is borrowed, returns `None`.
	pub fn as_mut(&mut self) -> Option<&mut T>
	where
		T::Owned: BorrowMut<T>,
	{
		match self {
			Mown::Owned(t) => Some(t.borrow_mut()),
			Mown::Borrowed(_) => None,
		}
	}

	pub fn into_owned(self) -> <T as Borrowed>::Owned
	where
		T: ToOwned<Owned = <T as Borrowed>::Owned>,
	{
		match self {
			Self::Borrowed(t) => t.to_owned(),
			Self::Owned(t) => t,
		}
	}
}

impl<'a, T: ?Sized + Borrowed> AsRef<T> for Mown<'a, T> {
	fn as_ref(&self) -> &T {
		match self {
			Mown::Owned(t) => t.borrow(),
			Mown::Borrowed(t) => t,
		}
	}
}

impl<'a, T: ?Sized + Borrowed> Deref for Mown<'a, T> {
	type Target = T;

	fn deref(&self) -> &T {
		self.as_ref()
	}
}

impl<'a, T: ?Sized + Borrowed + PartialEq> PartialEq for Mown<'a, T> {
	fn eq(&self, other: &Mown<'a, T>) -> bool {
		self.as_ref() == other.as_ref()
	}
}

impl<'a, T: ?Sized + Borrowed + Eq> Eq for Mown<'a, T> {}

impl<'a, T: ?Sized + Borrowed + PartialOrd> PartialOrd for Mown<'a, T> {
	fn partial_cmp(&self, other: &Mown<'a, T>) -> Option<Ordering> {
		self.as_ref().partial_cmp(other)
	}
}

impl<'a, T: ?Sized + Borrowed + Ord> Ord for Mown<'a, T> {
	fn cmp(&self, other: &Mown<'a, T>) -> Ordering {
		self.as_ref().cmp(other)
	}
}

impl<'a, T: ?Sized + Borrowed + Hash> Hash for Mown<'a, T> {
	fn hash<H: Hasher>(&self, hasher: &mut H) {
		self.as_ref().hash(hasher)
	}
}

impl<'a, T: ?Sized + Borrowed + Display> Display for Mown<'a, T> {
	fn fmt(&self, f: &mut Formatter) -> fmt::Result {
		self.as_ref().fmt(f)
	}
}

impl<'a, T: ?Sized + Borrowed + Debug> Debug for Mown<'a, T> {
	fn fmt(&self, f: &mut Formatter) -> fmt::Result {
		self.as_ref().fmt(f)
	}
}

impl<'a, T: ?Sized + Borrowed, Q: Borrow<T>> From<&'a Q> for Mown<'a, T> {
	fn from(r: &'a Q) -> Mown<'a, T> {
		Mown::Borrowed(r.borrow())
	}
}

/// Container for mutabily borrowed or owned values.
pub enum MownMut<'a, T: ?Sized + Borrowed> {
	/// Owned value.
	Owned(T::Owned),

	/// Borrowed value.
	Borrowed(&'a mut T),
}

impl<'a, T: ?Sized + Borrowed> MownMut<'a, T> {
	/// Checks if the value is owned.
	pub fn is_owned(&self) -> bool {
		match self {
			MownMut::Owned(_) => true,
			MownMut::Borrowed(_) => false,
		}
	}

	/// Checks if the value is borrowed.
	pub fn is_borrowed(&self) -> bool {
		match self {
			MownMut::Owned(_) => false,
			MownMut::Borrowed(_) => true,
		}
	}

	pub fn into_owned(self) -> <T as Borrowed>::Owned
	where
		T: ToOwned<Owned = <T as Borrowed>::Owned>,
	{
		match self {
			Self::Borrowed(t) => t.to_owned(),
			Self::Owned(t) => t,
		}
	}
}

impl<'a, T: ?Sized + Borrowed> AsRef<T> for MownMut<'a, T> {
	fn as_ref(&self) -> &T {
		match self {
			MownMut::Owned(t) => t.borrow(),
			MownMut::Borrowed(t) => t,
		}
	}
}

impl<'a, T: ?Sized + Borrowed> AsMut<T> for MownMut<'a, T>
where
	T::Owned: BorrowMut<T>,
{
	fn as_mut(&mut self) -> &mut T {
		match self {
			MownMut::Owned(t) => t.borrow_mut(),
			MownMut::Borrowed(t) => t,
		}
	}
}

impl<'a, T: ?Sized + Borrowed> Deref for MownMut<'a, T> {
	type Target = T;

	fn deref(&self) -> &T {
		self.as_ref()
	}
}

impl<'a, T: ?Sized + Borrowed> DerefMut for MownMut<'a, T>
where
	T::Owned: BorrowMut<T>,
{
	fn deref_mut(&mut self) -> &mut T {
		self.as_mut()
	}
}

impl<'a, T: ?Sized + Borrowed + PartialEq> PartialEq for MownMut<'a, T> {
	fn eq(&self, other: &MownMut<'a, T>) -> bool {
		self.as_ref() == other.as_ref()
	}
}

impl<'a, T: ?Sized + Borrowed + Eq> Eq for MownMut<'a, T> {}

impl<'a, T: ?Sized + Borrowed + PartialOrd> PartialOrd for MownMut<'a, T> {
	fn partial_cmp(&self, other: &MownMut<'a, T>) -> Option<Ordering> {
		self.as_ref().partial_cmp(other)
	}
}

impl<'a, T: ?Sized + Borrowed + Ord> Ord for MownMut<'a, T> {
	fn cmp(&self, other: &MownMut<'a, T>) -> Ordering {
		self.as_ref().cmp(other)
	}
}

impl<'a, T: ?Sized + Borrowed + Hash> Hash for MownMut<'a, T> {
	fn hash<H: Hasher>(&self, hasher: &mut H) {
		self.as_ref().hash(hasher)
	}
}

impl<'a, T: ?Sized + Borrowed + Display> Display for MownMut<'a, T> {
	fn fmt(&self, f: &mut Formatter) -> fmt::Result {
		self.as_ref().fmt(f)
	}
}

impl<'a, T: ?Sized + Borrowed + Debug> Debug for MownMut<'a, T> {
	fn fmt(&self, f: &mut Formatter) -> fmt::Result {
		self.as_ref().fmt(f)
	}
}

impl<'a, T: ?Sized + Borrowed, Q: BorrowMut<T>> From<&'a mut Q> for MownMut<'a, T> {
	fn from(r: &'a mut Q) -> MownMut<'a, T> {
		MownMut::Borrowed(r.borrow_mut())
	}
}
