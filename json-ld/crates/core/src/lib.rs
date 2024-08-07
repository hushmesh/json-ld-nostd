//! JSON-LD core types.

#![cfg_attr(not(feature = "std"), no_std)]

extern crate alloc;

extern crate thiserror_nostd_notrait as thiserror;

pub use json_ld_syntax::{Direction, LenientLangTag, LenientLangTagBuf, Nullable};

mod container;
pub mod context;
mod deserialization;
mod document;
pub mod flattening;
pub mod id;
mod indexed;
mod lang_string;
pub mod loader;
mod mode;
pub mod object;
pub mod print;
pub mod quad;
pub mod rdf;
mod serialization;
mod term;
mod ty;
pub mod utils;
pub mod warning;

pub use container::{Container, ContainerKind};
pub use context::Context;
pub use document::*;
pub use flattening::Flatten;
pub use id::*;
pub use indexed::*;
pub use lang_string::*;
pub use loader::*;
pub use mode::*;
pub use object::{IndexedNode, IndexedObject, Node, Nodes, Object, Objects, TryFromJson, Value};
pub use print::Print;
pub use quad::LdQuads;
pub use rdf::RdfQuads;
pub use term::*;
pub use ty::*;

pub struct Environment<'a, N, L> {
	pub vocabulary: &'a mut N,
	pub loader: &'a L,
}
