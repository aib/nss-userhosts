mod nss;

use std::os::raw::{c_char, c_int};
use nss::{hostent, nss_status};
use nss::nss_status_NSS_STATUS_UNAVAIL;

const AF_INET: c_int = 2;
const ENOENT: c_int = 2;

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
	_name: *const c_char, _type: c_int, _result_buf: *mut hostent,
	_buf: *mut c_char, _buflen: usize,
	errnop: *mut c_int, _h_errnop: *mut c_int,
) -> nss_status {
	*errnop = ENOENT;
	nss_status_NSS_STATUS_UNAVAIL
}
