use iref::{Iri, IriRef, Uri, UriRef};
use static_iref::{iri, iri_ref, uri, uri_ref};

const URI: &'static Uri = uri!("https://www.rust-lang.org/foo/bar#frag");
const URI_REF: &'static UriRef = uri_ref!("/foo/bar#frag");

const IRI: &'static Iri = iri!("https://www.rust-lang.org/foo/bar#frag");
const IRI_REF: &'static IriRef = iri_ref!("/foo/bar#frag");

#[test]
fn uri() {
	assert_eq!(
		URI,
		Uri::new("https://www.rust-lang.org/foo/bar#frag").unwrap()
	);
}

#[test]
fn uri_ref() {
	assert_eq!(URI_REF, UriRef::new("/foo/bar#frag").unwrap());
}

#[test]
fn iri() {
	assert_eq!(
		IRI,
		Iri::new("https://www.rust-lang.org/foo/bar#frag").unwrap()
	);
}

#[test]
fn iri_ref() {
	assert_eq!(IRI_REF, IriRef::new("/foo/bar#frag").unwrap());
}
