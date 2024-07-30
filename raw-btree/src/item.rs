use alloc::borrow::Borrow;
use core::cmp::Ordering;

#[derive(Debug)]
pub struct Item<K, V> {
	pub key: K,
	pub value: V,
}

impl<K, V> Item<K, V> {
	pub fn new(key: K, value: V) -> Self {
		Self { key, value }
	}

	pub fn key_cmp<Q>(&self, key: &Q) -> Ordering
	where
		K: Borrow<Q> + Ord,
		Q: Ord + ?Sized,
	{
		self.key.borrow().cmp(key)
	}
}

impl<K: PartialEq, V> PartialEq for Item<K, V> {
	fn eq(&self, other: &Self) -> bool {
		self.key == other.key
	}
}

impl<K: Eq, V> Eq for Item<K, V> {}

impl<K: PartialOrd, V> PartialOrd for Item<K, V> {
	fn partial_cmp(&self, other: &Self) -> Option<core::cmp::Ordering> {
		self.key.partial_cmp(&other.key)
	}
}

impl<K: Ord, V> Ord for Item<K, V> {
	fn cmp(&self, other: &Self) -> core::cmp::Ordering {
		self.key.cmp(&other.key)
	}
}

impl<K: core::fmt::Display, V: core::fmt::Display> core::fmt::Display for Item<K, V> {
	fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
		write!(f, "({}, {})", self.key, self.value)
	}
}
