mod nss;

use std::collections::HashMap;
use std::ffi::{c_char, c_int};
use std::ffi::CStr;
use std::net::{Ipv4Addr, Ipv6Addr};
use std::path::PathBuf;
use nss::{hostent, nss_status};
use nss::{NSS_STATUS_TRYAGAIN, NSS_STATUS_UNAVAIL, NSS_STATUS_NOTFOUND, NSS_STATUS_SUCCESS};

const AF_INET: c_int = 2;
const AF_INET6: c_int = 10;

const ENOENT: c_int = 2;
const EINVAL: c_int = 22;
const ERANGE: c_int = 34;

const HOST_NOT_FOUND: c_int = 1;
const NO_RECOVERY: c_int = 3;

struct HostsMap {
	pub ipv4: HashMap<String, Vec<Ipv4Addr>>,
	pub ipv6: HashMap<String, Vec<Ipv6Addr>>,
}

#[derive(Clone)]
enum HostAddress {
	Ipv4(Ipv4Addr),
	Ipv6(Ipv6Addr),
}

impl HostsMap {
	pub fn new() -> Self {
		Self { ipv4: HashMap::new(), ipv6: HashMap::new() }
	}

	pub fn add(&mut self, hostname: String, address: HostAddress) {
		match address {
			HostAddress::Ipv4(ipv4) => self.ipv4.entry(hostname).or_default().push(ipv4),
			HostAddress::Ipv6(ipv6) => self.ipv6.entry(hostname).or_default().push(ipv6),
		}
	}
}

fn load_hosts_map() -> HostsMap {
	let mut hosts_map = HostsMap::new();

	if let Ok(env_string) = std::env::var("USERHOSTS") {
		parse_hosts_string(&mut hosts_map, &env_string);
	}

	let userhosts_path = std::env::var("USERHOSTS_FILE")
		.map(PathBuf::from)
		.or_else(|_| {
			std::env::var("HOME").map(|home| PathBuf::from(home).join("hosts"))
		});

	if let Ok(userhosts_path) = userhosts_path {
		if let Ok(contents) = std::fs::read_to_string(userhosts_path) {
			parse_hosts_file(&mut hosts_map, &contents);
		}
	}

	hosts_map
}

fn parse_hosts_file(hosts_map: &mut HostsMap, contents: &str) {
	for line in contents.lines() {
		let uncommented = line.split_once('#').map(|pair| pair.0).unwrap_or(line);
		parse_hosts_line(hosts_map, uncommented);
	}
}

fn parse_hosts_string(hosts_map: &mut HostsMap, contents: &str) {
	for line in contents.split(';') {
		parse_hosts_line(hosts_map, line);
	}
}

fn parse_hosts_line(hosts_map: &mut HostsMap, line: &str) {
	let mut split = line.split_ascii_whitespace();

	let ip = match split.next() {
		Some(s) => s,
		None => return,
	};

	let address = {
		use std::str::FromStr;
		if let Ok(ipv4) = Ipv4Addr::from_str(ip) {
			HostAddress::Ipv4(ipv4)
		} else if let Ok(ipv6) = Ipv6Addr::from_str(ip) {
			HostAddress::Ipv6(ipv6)
		} else {
			return
		}
	};

	for hostname in split {
		hosts_map.add(hostname.to_owned(), address.clone());
	}
}

enum AddressList<'a> {
	Ipv4(&'a [Ipv4Addr]),
	Ipv6(&'a [Ipv6Addr]),
}

fn write_hostent(target: *mut hostent, buf: &mut [u8], name: &CStr, aliases: &[&CStr], addresses: AddressList) -> Option<()> {
	let mut buf_next_free = buf.as_mut_ptr();
	let mut buf_left = buf.len();
	let mut alloc = |size: usize| -> Option<&mut [u8]> {
		if buf_left < size {
			None
		} else {
			let slice = unsafe { std::slice::from_raw_parts_mut(buf_next_free, size) };
			buf_next_free = unsafe { buf_next_free.add(size) };
			buf_left -= size;
			Some(slice)
		}
	};

	let name = name.to_bytes_with_nul();
	let h_name = alloc(name.len())?;
	h_name.copy_from_slice(name);

	let mut alias_ptrs: Vec<*mut c_char> = Vec::new();
	for alias in aliases {
		let alias = alias.to_bytes_with_nul();
		let h_alias = alloc(alias.len())?;
		h_alias.copy_from_slice(alias);
		alias_ptrs.push(h_alias.as_mut_ptr().cast());
	}
	alias_ptrs.push(std::ptr::null_mut());

	let h_aliases = alloc(alias_ptrs.len() * std::mem::size_of::<*mut c_char>())?;
	unsafe {
		h_aliases.as_mut_ptr().copy_from(alias_ptrs.as_ptr().cast(), h_aliases.len());
	}

	let h_addrtype;
	let h_length;
	let mut addr_ptrs: Vec<*mut c_char> = Vec::new();

	match addresses {
		AddressList::Ipv4(addrs) => {
			h_addrtype = AF_INET;
			h_length = 4;

			for addr in addrs {
				let h_addr = alloc(4)?;
				h_addr.copy_from_slice(&addr.octets());
				addr_ptrs.push(h_addr.as_mut_ptr().cast());
			}
		}
		AddressList::Ipv6(addrs) => {
			h_addrtype = AF_INET6;
			h_length = 16;

			for addr in addrs {
				let h_addr = alloc(16)?;
				h_addr.copy_from_slice(&addr.octets());
				addr_ptrs.push(h_addr.as_mut_ptr().cast());
			}
		}
	}
	addr_ptrs.push(std::ptr::null_mut());

	let h_addr_list = alloc(addr_ptrs.len() * std::mem::size_of::<*mut c_char>())?;
	unsafe {
		h_addr_list.as_mut_ptr().copy_from(addr_ptrs.as_ptr().cast(), h_addr_list.len());
	}

	unsafe {
		(*target).h_name = h_name.as_mut_ptr().cast();
		(*target).h_aliases = h_aliases.as_mut_ptr().cast();
		(*target).h_addrtype = h_addrtype;
		(*target).h_length = h_length;
		(*target).h_addr_list = h_addr_list.as_mut_ptr().cast();
	}
	Some(())
}

#[no_mangle]
pub unsafe extern "C" fn _nss_userhosts_gethostbyname_r(
	name: *const c_char, result_buf: *mut hostent,
	buf: *mut c_char, buflen: usize,
	errnop: *mut c_int, h_errnop: *mut c_int,
) -> nss_status {
	_nss_userhosts_gethostbyname2_r(name, AF_INET, result_buf, buf, buflen, errnop, h_errnop)
}

#[no_mangle]
pub unsafe extern "C" fn _nss_userhosts_gethostbyname2_r(
	name: *const c_char, r#type: c_int, result_buf: *mut hostent,
	buf: *mut c_char, buflen: usize,
	errnop: *mut c_int, h_errnop: *mut c_int,
) -> nss_status {
	let not_found = || {
		*errnop = ENOENT;
		*h_errnop = HOST_NOT_FOUND;
		NSS_STATUS_NOTFOUND
	};

	let buffer_too_small = || {
		*errnop = ERANGE;
		NSS_STATUS_TRYAGAIN
	};

	let bad_type = || {
		*errnop = EINVAL;
		*h_errnop = NO_RECOVERY;
		NSS_STATUS_UNAVAIL
	};

	let success = || {
		*errnop = 0;
		*h_errnop = 0;
		NSS_STATUS_SUCCESS
	};

	let hosts_map = load_hosts_map();
	let name_cstr = CStr::from_ptr(name);
	let user_buf = std::slice::from_raw_parts_mut(buf.cast(), buflen);

	match r#type {
		AF_INET => {
			let aliases = [];
			match hosts_map.ipv4.get(name_cstr.to_string_lossy().as_ref()) {
				Some(ips) => {
					match write_hostent(result_buf, user_buf, name_cstr, &aliases, AddressList::Ipv4(ips)) {
						Some(_) => success(),
						None => buffer_too_small(),
					}
				}
				None => not_found(),
			}
		}
		AF_INET6 => {
			let aliases = [];
			match hosts_map.ipv6.get(name_cstr.to_string_lossy().as_ref()) {
				Some(ips) => {
					match write_hostent(result_buf, user_buf, name_cstr, &aliases, AddressList::Ipv6(ips)) {
						Some(_) => success(),
						None => buffer_too_small(),
					}
				}
				None => not_found(),
			}
		}
		_ => bad_type(),
	}
}
