/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use std::ffi::{CStr, CString};
use std::os::raw::c_char;

pub fn c_char_to_string<'a>(cchar: *const c_char) -> &'a str {
    let c_str = unsafe { CStr::from_ptr(cchar) };
    c_str.to_str().unwrap_or("")
}

pub fn string_to_c_char<T>(r_string: T) -> *mut c_char
where
    T: Into<String>,
{
    CString::new(r_string.into()).unwrap().into_raw()
}
