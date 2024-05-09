fn main() {
	println!("cargo:rustc-cdylib-link-arg=-Wl,-soname,libnss_userhosts.so.2");
}
