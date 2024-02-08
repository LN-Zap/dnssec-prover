//! WASM-compatible verification wrappers

use dnssec_prover::ser::parse_rr_stream;
use dnssec_prover::validation::{verify_rr_stream, ValidationError};

use wasm_bindgen::prelude::wasm_bindgen;

use core::fmt::Write;

#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

#[wasm_bindgen]
/// Verifies an RFC 9102-formatted proof and returns the [`VerifiedRRStream`] in JSON form.
pub fn verify_byte_stream(stream: Vec<u8>) -> String {
	match do_verify_byte_stream(stream) {
		Ok(r) => r,
		Err(e) => format!("{{\"error\":\"{:?}\"}}", e),
	}
}

fn do_verify_byte_stream(stream: Vec<u8>) -> Result<String, ValidationError> {
	let rrs = parse_rr_stream(&stream).map_err(|()| ValidationError::Invalid)?;
	let verified_rrs = verify_rr_stream(&rrs)?;
	let mut resp = String::new();
	write!(&mut resp, "{}",
		format_args!("{{\"valid_from\": {}, \"expires\": {}, \"max_cache_ttl\": {}, \"verified_rrs\": [",
		verified_rrs.valid_from, verified_rrs.expires, verified_rrs.max_cache_ttl)
	).expect("Write to a String shouldn't fail");
	for (idx, rr) in verified_rrs.verified_rrs.iter().enumerate() {
		write!(&mut resp, "{}{}", if idx != 0 { ", " } else { "" }, rr.json())
			.expect("Write to a String shouldn't fail");
	}
	resp += "]}";
	Ok(resp)
}