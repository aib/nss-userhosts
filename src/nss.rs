use std::ffi::{c_char, c_int};

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct hostent {
    pub h_name: *mut c_char,
    pub h_aliases: *mut *mut c_char,
    pub h_addrtype: c_int,
    pub h_length: c_int,
    pub h_addr_list: *mut *mut c_char,
}

#[allow(non_camel_case_types)]
pub type nss_status = c_int;

pub const NSS_STATUS_TRYAGAIN: nss_status = -2;
pub const NSS_STATUS_UNAVAIL: nss_status = -1;
pub const NSS_STATUS_NOTFOUND: nss_status = 0;
pub const NSS_STATUS_SUCCESS: nss_status = 1;
