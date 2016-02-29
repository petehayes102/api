// Copyright 2015 Intecture Developers. See the COPYRIGHT file at the
// top-level directory of this distribution and at
// https://intecture.io/COPYRIGHT.
//
// Licensed under the Mozilla Public License 2.0 <LICENSE or
// https://www.tldrlegal.com/l/mpl-2.0>. This file may not be copied,
// modified, or distributed except according to those terms.

//! FFI interface for Target Data

use libc::c_char;
use std::convert::From;
use std::ffi::{CStr, CString};
use std::str;
use super::Item;

impl <'a>From<Item<&'a str>> for Item<*mut c_char> {
    fn from(item: Item<&str>) -> Item<*mut c_char> {
        Item {
            centos: if let Some(option) = item.centos {
                Some(CString::new(option).unwrap().into_raw())
            } else {
                None
            },
            debian: if let Some(option) = item.debian {
                Some(CString::new(option).unwrap().into_raw())
            } else {
                None
            },
            default: if let Some(option) = item.default {
                Some(CString::new(option).unwrap().into_raw())
            } else {
                None
            },
            fedora: if let Some(option) = item.fedora {
                Some(CString::new(option).unwrap().into_raw())
            } else {
                None
            },
            freebsd: if let Some(option) = item.freebsd {
                Some(CString::new(option).unwrap().into_raw())
            } else {
                None
            },
            linux: if let Some(option) = item.linux {
                Some(CString::new(option).unwrap().into_raw())
            } else {
                None
            },
            macos: if let Some(option) = item.macos {
                Some(CString::new(option).unwrap().into_raw())
            } else {
                None
            },
            redhat: if let Some(option) = item.redhat {
                Some(CString::new(option).unwrap().into_raw())
            } else {
                None
            },
            ubuntu: if let Some(option) = item.ubuntu {
                Some(CString::new(option).unwrap().into_raw())
            } else {
                None
            },
            unix: if let Some(option) = item.unix {
                Some(CString::new(option).unwrap().into_raw())
            } else {
                None
            },
        }
    }
}

impl <'a>From<Item<*mut c_char>> for Item<&'a str> {
    fn from(item: Item<*mut c_char>) -> Item<&'a str> {
        Item {
            centos: if let Some(option) = item.centos {
                Some(str::from_utf8(unsafe { CStr::from_ptr(option) }.to_bytes()).unwrap())
            } else {
                None
            },
            debian: if let Some(option) = item.debian {
                Some(str::from_utf8(unsafe { CStr::from_ptr(option) }.to_bytes()).unwrap())
            } else {
                None
            },
            default: if let Some(option) = item.default {
                Some(str::from_utf8(unsafe { CStr::from_ptr(option) }.to_bytes()).unwrap())
            } else {
                None
            },
            fedora: if let Some(option) = item.fedora {
                Some(str::from_utf8(unsafe { CStr::from_ptr(option) }.to_bytes()).unwrap())
            } else {
                None
            },
            freebsd: if let Some(option) = item.freebsd {
                Some(str::from_utf8(unsafe { CStr::from_ptr(option) }.to_bytes()).unwrap())
            } else {
                None
            },
            linux: if let Some(option) = item.linux {
                Some(str::from_utf8(unsafe { CStr::from_ptr(option) }.to_bytes()).unwrap())
            } else {
                None
            },
            macos: if let Some(option) = item.macos {
                Some(str::from_utf8(unsafe { CStr::from_ptr(option) }.to_bytes()).unwrap())
            } else {
                None
            },
            redhat: if let Some(option) = item.redhat {
                Some(str::from_utf8(unsafe { CStr::from_ptr(option) }.to_bytes()).unwrap())
            } else {
                None
            },
            ubuntu: if let Some(option) = item.ubuntu {
                Some(str::from_utf8(unsafe { CStr::from_ptr(option) }.to_bytes()).unwrap())
            } else {
                None
            },
            unix: if let Some(option) = item.unix {
                Some(str::from_utf8(unsafe { CStr::from_ptr(option) }.to_bytes()).unwrap())
            } else {
                None
            },
        }
    }
}
