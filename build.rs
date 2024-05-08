use std::env;
use std::path::PathBuf;

fn main() {
	let out_dir = PathBuf::from(
		env::var("OUT_DIR").expect("OUT_DIR not set")
	);

	bindgen::Builder::default()
		.header("src/nss/nss.h")
		.generate()
		.expect("Could not generate nss bindings")
		.write_to_file(out_dir.join("nss.rs"))
		.expect("Could not write nss bindings");
}
