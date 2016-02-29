// Copyright 2015 Intecture Developers. See the COPYRIGHT file at the
// top-level directory of this distribution and at
// https://intecture.io/COPYRIGHT.
//
// Licensed under the Mozilla Public License 2.0 <LICENSE or
// https://www.tldrlegal.com/l/mpl-2.0>. This file may not be copied,
// modified, or distributed except according to those terms.

//! FFI interface for Host

use Telemetry;
#[cfg(feature = "remote-run")]
use libc::{c_char, c_void, uint32_t};
use std::convert;
#[cfg(feature = "remote-run")]
use std::{ptr, str};
#[cfg(feature = "remote-run")]
use std::ffi::{CStr, CString};
use super::*;
use telemetry::ffi::Ffi__Telemetry;
#[cfg(feature = "remote-run")]
use zmq;

#[cfg(feature = "remote-run")]
#[repr(C)]
pub struct Ffi__Host {
    #[cfg(feature = "remote-run")]
    hostname: Option<*mut c_char>,
    #[cfg(feature = "remote-run")]
    api_sock: Option<*mut c_void>,
    #[cfg(feature = "remote-run")]
    upload_sock: Option<*mut c_void>,
    #[cfg(feature = "remote-run")]
    download_port: Option<uint32_t>,
    telemetry: Ffi__Telemetry,
}

impl convert::From<Host> for Ffi__Host {
    #[cfg(feature = "local-run")]
    #[allow(unused_variables)]
    fn from(host: Host) -> Ffi__Host {
        Ffi__Host {
            telemetry: Ffi__Telemetry::from(host.telemetry),
        }
    }

    #[cfg(feature = "remote-run")]
    fn from(host: Host) -> Ffi__Host {
        Ffi__Host {
            hostname: if let Some(hostname) = host.hostname {
                Some(CString::new(hostname).unwrap().into_raw())
            } else {
                None
            },
            api_sock: if let Some(mut sock) = host.api_sock {
                Some(sock.to_raw())
            } else {
                None
            },
            upload_sock: if let Some(mut sock) = host.upload_sock {
                Some(sock.to_raw())
            } else {
                None
            },
            download_port: host.download_port,
            telemetry: Ffi__Telemetry::from(host.telemetry),
        }
    }
}

impl convert::From<Ffi__Host> for Host {
    #[cfg(feature = "local-run")]
    #[allow(unused_variables)]
    fn from(ffi_host: Ffi__Host) -> Host {
        Host {
            telemetry: Telemetry::from(ffi_host.telemetry),
        }
    }

    #[cfg(feature = "remote-run")]
    fn from(ffi_host: Ffi__Host) -> Host {
        Host {
            hostname: if let Some(hostname) = ffi_host.hostname {
                Some(str::from_utf8(unsafe { CStr::from_ptr(hostname).to_bytes() }).unwrap().to_string())
            } else {
                None
            },
            api_sock: if let Some(sock) = ffi_host.api_sock {
                Some(zmq::Socket::from_raw(sock))
            } else {
                None
            },
            upload_sock: if let Some(sock) = ffi_host.upload_sock {
                Some(zmq::Socket::from_raw(sock))
            } else {
                None
            },
            download_port: ffi_host.download_port,
            telemetry: Telemetry::from(ffi_host.telemetry),
        }
    }
}

#[no_mangle]
pub extern "C" fn host_new() -> Ffi__Host {
    Ffi__Host::from(Host::new())
}

#[cfg(feature = "remote-run")]
#[no_mangle]
pub extern "C" fn host_connect(ffi_host_ptr: *mut Ffi__Host,
                               ip: *const c_char,
                               api_port: uint32_t,
                               upload_port: uint32_t,
                               download_port: uint32_t) {
    let slice = unsafe { CStr::from_ptr(ip) };
    let ip_str = str::from_utf8(slice.to_bytes()).unwrap();

    let mut host = Host::from(unsafe { ptr::read(ffi_host_ptr) });
    host.connect(ip_str, api_port, upload_port, download_port).unwrap();

    unsafe { ptr::write(&mut *ffi_host_ptr, Ffi__Host::from(host)); }
}

#[cfg(feature = "remote-run")]
#[no_mangle]
pub extern "C" fn host_close(ffi_host_ptr: *mut Ffi__Host) {
    let mut host = Host::from(unsafe { ptr::read(ffi_host_ptr) });
    host.close().unwrap();
}

#[no_mangle]
pub extern "C" fn host_telemetry(ffi_host_ptr: *mut Ffi__Host) -> Ffi__Telemetry {
    let host = Host::from(unsafe { ptr::read(ffi_host_ptr) });
    Ffi__Telemetry::from(host.telemetry)
}

#[cfg(test)]
mod tests {
    use {Host, Telemetry};
    #[cfg(feature = "remote-run")]
    use std::ffi::CString;
    use super::*;
    use telemetry::ffi::Ffi__Telemetry;
    #[cfg(feature = "remote-run")]
    use zmq;

    #[test]
    fn test_convert_host() {
        let host = Host::new();
        Ffi__Host::from(host);
    }

    #[cfg(feature = "remote-run")]
    #[test]
    fn test_convert_host_connected() {
        let mut host = Host::new();
        assert!(host.connect("127.0.0.1", 7101, 7102, 7103).is_ok());
        Ffi__Host::from(host);
    }

    #[cfg(feature = "remote-run")]
    #[test]
    fn test_convert_ffi_host() {
        let mut ctx = zmq::Context::new();
        let mut sock = ctx.socket(zmq::REQ).unwrap();

        let ffi_host = Ffi__Host {
            hostname: Some(CString::new("localhost").unwrap().into_raw()),
            api_sock: Some(sock.to_raw()),
            upload_sock: None,
            download_port: None,
            telemetry: Ffi__Telemetry::from(Telemetry::new()),
        };

        Host::from(ffi_host);
    }

    #[cfg(feature = "remote-run")]
    #[test]
    fn test_host_fns() {
        let mut host = host_new();
        host_connect(&mut host as *mut Ffi__Host,
                     CString::new("localhost").unwrap().as_ptr(),
                     7101,
                     7102,
                     7103);
        host_close(&mut host as *mut Ffi__Host);
    }
}
