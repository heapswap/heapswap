use std::marker::PhantomData;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsValue;

#[wasm_bindgen]
pub fn collect_numbers(
	some_iterable: &JsValue,
) -> Result<js_sys::Array, JsValue> {
	let nums = js_sys::Array::new();

	let iterator = js_sys::try_iter(some_iterable)?
		.ok_or_else(|| "need to pass iterable JS values!")?;

	for x in iterator {
		let x = x?;
		nums.push(&x);
	}

	Ok(nums)
}
