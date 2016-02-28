// Copyright 2015 Intecture Developers. See the COPYRIGHT file at the
// top-level directory of this distribution and at
// https://intecture.io/COPYRIGHT.
//
// Licensed under the Mozilla Public License 2.0 <LICENSE or
// https://www.tldrlegal.com/l/mpl-2.0>. This file may not be copied,
// modified, or distributed except according to those terms.

//! FFI interface for Telemetry

use Host;
use host::ffi::Ffi__Host;
use libc::{c_char, c_float, size_t, uint32_t, uint64_t};
use std::{convert, mem, ptr};
use std::ffi::{CStr, CString};
use super::*;

#[repr(C)]
pub struct Ffi__Telemetry {
    pub cpu: Option<Ffi__Cpu>,
    pub fs: Option<Ffi__Array<Ffi__FsMount>>,
    pub hostname: Option<*mut c_char>,
    pub memory: Option<uint64_t>,
    pub net: Option<Ffi__Array<Ffi__Netif>>,
    pub os: Option<Ffi__Os>,
}

impl convert::From<Telemetry> for Ffi__Telemetry {
    fn from(telemetry: Telemetry) -> Ffi__Telemetry {
        Ffi__Telemetry {
            cpu: if let Some(cpu) = telemetry.cpu {
                Some(Ffi__Cpu::from(cpu))
            } else {
                None
            },
            fs: if let Some(mut fs) = telemetry.fs {
                let ffi_fs: Vec<_> = fs.drain(..).map(|mount| Ffi__FsMount::from(mount)).collect();
                Some(Ffi__Array::from(ffi_fs))
            } else {
                None
            },
            hostname: if let Some(hostname) = telemetry.hostname {
                Some(CString::new(hostname).unwrap().into_raw())
            } else {
                None
            },
            memory: telemetry.memory,
            net: if let Some(mut net) = telemetry.net {
                let ffi_net: Vec<_> = net.drain(..).map(|netif| Ffi__Netif::from(netif)).collect();
                Some(Ffi__Array::from(ffi_net))
            } else {
                None
            },
            os: if let Some(os) = telemetry.os {
                Some(Ffi__Os::from(os))
            } else {
                None
            }
        }
    }
}

impl convert::From<Ffi__Telemetry> for Telemetry {
    fn from(ffi_telemetry: Ffi__Telemetry) -> Telemetry {
        Telemetry {
            cpu: if let Some(cpu) = ffi_telemetry.cpu {
                Some(Cpu::from(cpu))
            } else {
                None
            },
            fs: if let Some(ffi_fs) = ffi_telemetry.fs {
                let mut fs_vec = unsafe { Vec::from_raw_parts(ffi_fs.ptr, ffi_fs.length, ffi_fs.capacity) };
                let fs: Vec<_> = fs_vec.drain(..).map(|mount| FsMount::from(mount)).collect();
                Some(fs)
            } else {
                None
            },
            hostname: if let Some(hostname) = ffi_telemetry.hostname {
                Some(unsafe { CString::from_raw(hostname) }.to_str().unwrap().to_string())
            } else {
                None
            },
            memory: ffi_telemetry.memory,
            net: if let Some(ffi_net) = ffi_telemetry.net {
                let mut net_vec = unsafe { Vec::from_raw_parts(ffi_net.ptr, ffi_net.length, ffi_net.capacity) };
                let net: Vec<_> = net_vec.drain(..).map(|netif| Netif::from(netif)).collect();
                Some(net)
            } else {
                None
            },
            os: if let Some(os) = ffi_telemetry.os {
                Some(Os::from(os))
            } else {
                None
            },
        }
    }
}

#[repr(C)]
#[derive(Debug)]
pub struct Ffi__Cpu {
    pub vendor: *mut c_char,
    pub brand_string: *mut c_char,
    pub cores: uint32_t,
}

impl convert::From<Cpu> for Ffi__Cpu {
    fn from(cpu: Cpu) -> Ffi__Cpu {
        Ffi__Cpu {
            vendor: CString::new(cpu.vendor).unwrap().into_raw(),
            brand_string: CString::new(cpu.brand_string).unwrap().into_raw(),
            cores: cpu.cores as uint32_t,
        }
    }
}

impl convert::From<Ffi__Cpu> for Cpu {
    fn from(ffi_cpu: Ffi__Cpu) -> Cpu {
        Cpu {
            vendor: unsafe { CString::from_raw(ffi_cpu.vendor) }.to_str().unwrap().to_string(),
            brand_string: unsafe { CString::from_raw(ffi_cpu.brand_string) }.to_str().unwrap().to_string(),
            cores: ffi_cpu.cores as u32,
        }
    }
}

#[repr(C)]
pub struct Ffi__FsMount {
    pub filesystem: *mut c_char,
    pub mountpoint: *mut c_char,
    pub size: uint64_t,
    pub used: uint64_t,
    pub available: uint64_t,
    pub capacity: c_float,
//    pub inodes_used: uint64_t,
//    pub inodes_available: uint64_t,
//    pub inodes_capacity: c_float,
}

impl convert::From<FsMount> for Ffi__FsMount {
    fn from(mount: FsMount) -> Ffi__FsMount {
        Ffi__FsMount {
            filesystem: CString::new(mount.filesystem).unwrap().into_raw(),
            mountpoint: CString::new(mount.mountpoint).unwrap().into_raw(),
            size: mount.size as uint64_t,
            used: mount.used as uint64_t,
            available: mount.available as uint64_t,
            capacity: mount.capacity as c_float,
//            inodes_used: mount.inodes_used as uint64_t,
//            inodes_available: mount.inodes_available as uint64_t,
//            inodes_capacity: mount.inodes_capacity as c_float,
        }
    }
}

impl convert::From<Ffi__FsMount> for FsMount {
    fn from(ffi_mount: Ffi__FsMount) -> FsMount {
        FsMount {
            filesystem: unsafe { CString::from_raw(ffi_mount.filesystem) }.to_str().unwrap().to_string(),
            mountpoint: unsafe { CString::from_raw(ffi_mount.mountpoint) }.to_str().unwrap().to_string(),
            size: ffi_mount.size as u64,
            used: ffi_mount.used as u64,
            available: ffi_mount.available as u64,
            capacity: ffi_mount.capacity as f32,
//            inodes_used: ffi_mount.inodes_used as u64,
//            inodes_available: ffi_mount.inodes_available as u64,
//            inodes_capacity: ffi_mount.inodes_capacity as f32,
        }
    }
}

#[repr(C)]
pub struct Ffi__Netif {
    pub interface: *mut c_char,
    pub mac: *mut c_char,
    pub inet: Ffi__NetifIPv4,
    pub inet6: Ffi__NetifIPv6,
    pub status: *mut c_char,
}

impl convert::From<Netif> for Ffi__Netif {
    fn from(netif: Netif) -> Ffi__Netif {
        Ffi__Netif {
            interface: CString::new(netif.interface).unwrap().into_raw(),
            mac: if netif.mac.is_some() {
                    CString::new(netif.mac.unwrap()).unwrap().into_raw()
                } else {
                    CString::new("").unwrap().into_raw()
                },
            inet: if netif.inet.is_some() {
                    Ffi__NetifIPv4::from(netif.inet.unwrap())
                } else {
                    Ffi__NetifIPv4::from(NetifIPv4 {
                        address: String::new(),
                        netmask: String::new(),
                    })
                },
            inet6: if netif.inet6.is_some() {
                    Ffi__NetifIPv6::from(netif.inet6.unwrap())
                } else {
                    Ffi__NetifIPv6::from(NetifIPv6 {
                        address: String::new(),
                        prefixlen: 0,
                        scopeid: None,
                    })
                },
            status: if netif.status.is_some() {
                    match netif.status.unwrap() {
                        NetifStatus::Active => CString::new("Active").unwrap().into_raw(),
                        NetifStatus::Inactive => CString::new("Inactive").unwrap().into_raw(),
                    }
                } else {
                    CString::new("").unwrap().into_raw()
                },
        }
    }
}

impl convert::From<Ffi__Netif> for Netif {
    fn from(ffi_netif: Ffi__Netif) -> Netif {
        Netif {
            interface: unsafe { CStr::from_ptr(ffi_netif.interface) }.to_str().unwrap().to_string(),
            mac: {
                let mac = unsafe { CStr::from_ptr(ffi_netif.mac) }.to_str().unwrap();
                if mac == "" {
                    None
                } else {
                    Some(mac.to_string())
                }
            },
            inet: {
                let ipv4 = NetifIPv4::from(ffi_netif.inet);
                if ipv4.address == "" {
                    None
                } else {
                    Some(ipv4)
                }
            },
            inet6: {
                let ipv6 = NetifIPv6::from(ffi_netif.inet6);
                if ipv6.address == "" {
                    None
                } else {
                    Some(ipv6)
                }
            },
            status: {
                let status = unsafe { CStr::from_ptr(ffi_netif.status) }.to_str().unwrap();
                match status {
                    "Active" => Some(NetifStatus::Active),
                    "Inactive" => Some(NetifStatus::Inactive),
                    _ => None,
                }
            }
        }
    }
}

#[repr(C)]
pub struct Ffi__NetifIPv4 {
    pub address: *mut c_char,
    pub netmask: *mut c_char,
}

impl convert::From<NetifIPv4> for Ffi__NetifIPv4 {
    fn from(netif: NetifIPv4) -> Ffi__NetifIPv4 {
        Ffi__NetifIPv4 {
            address: CString::new(netif.address).unwrap().into_raw(),
            netmask: CString::new(netif.netmask).unwrap().into_raw(),
        }
    }
}

impl convert::From<Ffi__NetifIPv4> for NetifIPv4 {
    fn from(ffi_netif: Ffi__NetifIPv4) -> NetifIPv4 {
        NetifIPv4 {
            address: unsafe { CStr::from_ptr(ffi_netif.address) }.to_str().unwrap().to_string(),
            netmask: unsafe { CStr::from_ptr(ffi_netif.netmask) }.to_str().unwrap().to_string(),
        }
    }
}

#[repr(C)]
pub struct Ffi__NetifIPv6 {
    pub address: *mut c_char,
    pub prefixlen: uint32_t,
    pub scopeid: *mut c_char,
}

impl convert::From<NetifIPv6> for Ffi__NetifIPv6 {
    fn from(netif: NetifIPv6) -> Ffi__NetifIPv6 {
        Ffi__NetifIPv6 {
            address: CString::new(netif.address).unwrap().into_raw(),
            prefixlen: netif.prefixlen as uint32_t,
            scopeid: if netif.scopeid.is_some() {
                CString::new(netif.scopeid.unwrap()).unwrap().into_raw()
            } else {
                CString::new("").unwrap().into_raw()
            },
        }
    }
}

impl convert::From<Ffi__NetifIPv6> for NetifIPv6 {
    fn from(netif: Ffi__NetifIPv6) -> NetifIPv6 {
        NetifIPv6 {
            address: unsafe { CStr::from_ptr(netif.address) }.to_str().unwrap().to_string(),
            prefixlen: netif.prefixlen as u8,
            scopeid: {
                let scopeid = unsafe { CStr::from_ptr(netif.scopeid) }.to_str().unwrap().to_string();
                if scopeid == "" {
                    None
                } else {
                    Some(scopeid)
                }
            }
        }
    }
}

#[repr(C)]
pub struct Ffi__Os {
    pub arch: *mut c_char,
    pub family: *mut c_char,
    pub platform: OsPlatform,
    pub version: *mut c_char,
}

impl convert::From<Os> for Ffi__Os {
    fn from(os: Os) -> Ffi__Os {
        Ffi__Os {
            arch: CString::new(os.arch).unwrap().into_raw(),
            family: CString::new(os.family).unwrap().into_raw(),
            platform: os.platform,
            version: CString::new(os.version).unwrap().into_raw(),
        }
    }
}

impl convert::From<Ffi__Os> for Os {
    fn from(os: Ffi__Os) -> Os {
        Os {
            arch: unsafe { CStr::from_ptr(os.arch) }.to_str().unwrap().to_string(),
            family: unsafe { CStr::from_ptr(os.family) }.to_str().unwrap().to_string(),
            platform: os.platform,
            version: unsafe { CStr::from_ptr(os.version) }.to_str().unwrap().to_string(),
        }
    }
}

#[repr(C)]
pub struct Ffi__Array<T> {
    pub ptr: *mut T,
    pub length: size_t,
    pub capacity: size_t,
}

impl <T>convert::From<Vec<T>> for Ffi__Array<T> {
    fn from(item: Vec<T>) -> Ffi__Array<T> {
        let mut item = item;

        item.shrink_to_fit();

        let ffi_item = Ffi__Array {
            ptr: item.as_mut_ptr(),
            length: item.len() as size_t,
            capacity: item.capacity() as size_t,
        };

        mem::forget(item);

        ffi_item
    }
}

#[no_mangle]
pub extern "C" fn telemetry_new() -> Ffi__Telemetry {
    Ffi__Telemetry {
        cpu: None,
        fs: None,
        hostname: None,
        memory: None,
        net: None,
        os: None,
    }
}

#[no_mangle]
pub extern "C" fn telemetry_cpu<'a>(ffi_telemetry_ptr: *mut Ffi__Telemetry, ffi_host_ptr: *mut Ffi__Host) -> Option<Ffi__Cpu> {
    let mut host = Host::from(unsafe { ptr::read(ffi_host_ptr) });
    let mut telemetry = Telemetry::from(unsafe { ptr::read(ffi_telemetry_ptr) });

    telemetry.get_cpu(&mut host).unwrap();
    let ffi_telemetry = Ffi__Telemetry::from(telemetry);

    // Write mutated Telemetry state back to pointer
    unsafe { ptr::write(ffi_telemetry_ptr, ffi_telemetry); }

    // Convert ZMQ socket to raw to avoid destructor closing sock
    Ffi__Host::from(host);

    unsafe { ptr::read(ffi_telemetry_ptr) }.cpu
}

#[no_mangle]
pub extern "C" fn telemetry_fs(ffi_telemetry_ptr: *mut Ffi__Telemetry, ffi_host_ptr: *mut Ffi__Host) -> Option<Ffi__Array<Ffi__FsMount>> {
    let mut host = Host::from(unsafe { ptr::read(ffi_host_ptr) });
    let mut telemetry = Telemetry::from(unsafe { ptr::read(ffi_telemetry_ptr) });

    telemetry.get_fs(&mut host).unwrap();
    let ffi_telemetry = Ffi__Telemetry::from(telemetry);

    // Write mutated Telemetry state back to pointer
    unsafe { ptr::write(&mut *ffi_telemetry_ptr, ffi_telemetry); }

    // Convert ZMQ socket to raw to avoid destructor closing sock
    Ffi__Host::from(host);

    unsafe { ptr::read(ffi_telemetry_ptr) }.fs
}

#[no_mangle]
pub extern "C" fn telemetry_hostname(ffi_telemetry_ptr: *mut Ffi__Telemetry, ffi_host_ptr: *mut Ffi__Host) -> *const c_char {
    let mut host = Host::from(unsafe { ptr::read(ffi_host_ptr) });
    let mut telemetry = Telemetry::from(unsafe { ptr::read(ffi_telemetry_ptr) });

    telemetry.get_hostname(&mut host).unwrap();
    let ffi_telemetry = Ffi__Telemetry::from(telemetry);
    let ffi_hostname = ffi_telemetry.hostname.unwrap();

    // Write mutated Telemetry state back to pointer
    unsafe { ptr::write(&mut *ffi_telemetry_ptr, ffi_telemetry); }

    // Convert ZMQ socket to raw to avoid destructor closing sock
    Ffi__Host::from(host);

    ffi_hostname
}

#[no_mangle]
pub extern "C" fn telemetry_memory(ffi_telemetry_ptr: *mut Ffi__Telemetry, ffi_host_ptr: *mut Ffi__Host) -> uint64_t {
    let mut host = Host::from(unsafe { ptr::read(ffi_host_ptr) });
    let mut telemetry = Telemetry::from(unsafe { ptr::read(ffi_telemetry_ptr) });

    let memory = telemetry.get_memory(&mut host).unwrap();
    let ffi_telemetry = Ffi__Telemetry::from(telemetry);

    // Write mutated Telemetry state back to pointer
    unsafe { ptr::write(&mut *ffi_telemetry_ptr, ffi_telemetry); }

    // Convert ZMQ socket to raw to avoid destructor closing sock
    Ffi__Host::from(host);

    memory
}

#[no_mangle]
pub extern "C" fn telemetry_net(ffi_telemetry_ptr: *mut Ffi__Telemetry, ffi_host_ptr: *mut Ffi__Host) -> Option<Ffi__Array<Ffi__Netif>> {
    let mut host = Host::from(unsafe { ptr::read(ffi_host_ptr) });
    let mut telemetry = Telemetry::from(unsafe { ptr::read(ffi_telemetry_ptr) });

    telemetry.get_net(&mut host).unwrap();
    let ffi_telemetry = Ffi__Telemetry::from(telemetry);

    // Write mutated Telemetry state back to pointer
    unsafe { ptr::write(&mut *ffi_telemetry_ptr, ffi_telemetry); }

    // Convert ZMQ socket to raw to avoid destructor closing sock
    Ffi__Host::from(host);

    unsafe { ptr::read(ffi_telemetry_ptr) }.net
}

#[no_mangle]
pub extern "C" fn telemetry_os(ffi_telemetry_ptr: *mut Ffi__Telemetry, ffi_host_ptr: *mut Ffi__Host) -> Option<Ffi__Os> {
    let mut host = Host::from(unsafe { ptr::read(ffi_host_ptr) });
    let mut telemetry = Telemetry::from(unsafe { ptr::read(ffi_telemetry_ptr) });

    telemetry.get_os(&mut host).unwrap();
    let ffi_telemetry = Ffi__Telemetry::from(telemetry);;

    // Write mutated Telemetry state back to pointer
    unsafe { ptr::write(&mut *ffi_telemetry_ptr, ffi_telemetry); }

    // Convert ZMQ socket to raw to avoid destructor closing sock
    Ffi__Host::from(host);

    unsafe { ptr::read(ffi_telemetry_ptr) }.os
}

#[no_mangle]
pub extern "C" fn telemetry_free(ffi_telemetry_ptr: *mut Ffi__Telemetry) {
    // Once converted from raw pointers to Rust pointers, we can just
    // let the value fall out of scope to free.
    Telemetry::from(unsafe { ptr::read(ffi_telemetry_ptr) });
}

#[cfg(test)]
mod tests {
    #[cfg(feature = "remote-run")]
    use Host;
    #[cfg(feature = "remote-run")]
    use host::ffi::Ffi__Host;
    use libc::{c_float, size_t, uint32_t, uint64_t};
    #[cfg(feature = "remote-run")]
    use rustc_serialize::json;
    use std::ffi::{CStr, CString};
    use std::mem;
    #[cfg(feature = "remote-run")]
    use std::{str, thread};
    use super::*;
    use super::super::*;
    #[cfg(feature = "remote-run")]
    use zmq;

    #[test]
    fn test_convert_telemetry() {
        Ffi__Telemetry::from(create_telemetry());
    }

    #[test]
    fn test_convert_ffi_telemetry() {
        Telemetry::from(create_ffi_telemetry());
    }

    #[test]
    fn test_telemetry_new() {
        let telemetry = telemetry_new();
        assert!(telemetry.cpu.is_none());
    }

    #[cfg(feature = "remote-run")]
    #[test]
    fn test_telemetry_cpu() {
        let mut ctx = zmq::Context::new();

        let mut agent_sock = ctx.socket(zmq::REP).unwrap();
        agent_sock.bind("inproc://test").unwrap();

        let agent_mock = thread::spawn(move || {
            assert_eq!("telemetry::cpu", agent_sock.recv_string(0).unwrap().unwrap());
            assert_eq!(agent_sock.get_rcvmore().unwrap(), false);

            let cpu = Cpu {
                vendor: "moo".to_string(),
                brand_string: "Moo Cow Super Fun Happy CPU".to_string(),
                cores: 100,
            };

            agent_sock.send_str("Ok", zmq::SNDMORE).unwrap();
            agent_sock.send_str(&json::encode(&cpu).unwrap(), 0).unwrap();
        });

        let mut sock = ctx.socket(zmq::REQ).unwrap();
        sock.connect("inproc://test").unwrap();

        let mut host = Ffi__Host::from(Host::test_new(None, Some(sock), None, None));
        let mut telemetry = telemetry_new();
        let cpu = telemetry_cpu(&mut telemetry as *mut Ffi__Telemetry, &mut host as *mut Ffi__Host);

        assert_eq!(unsafe { str::from_utf8(CStr::from_ptr(cpu.unwrap().vendor).to_bytes()).unwrap() }, "moo");
        assert_eq!(unsafe { str::from_utf8(CStr::from_ptr(telemetry.cpu.unwrap().vendor).to_bytes()).unwrap() }, "moo");

        Host::from(host);

        agent_mock.join().unwrap();
    }

    #[cfg(feature = "remote-run")]
    #[test]
    fn test_telemetry_fs() {
        let mut ctx = zmq::Context::new();

        let mut agent_sock = ctx.socket(zmq::REP).unwrap();
        agent_sock.bind("inproc://test").unwrap();

        let agent_mock = thread::spawn(move || {
            assert_eq!("telemetry::fs", agent_sock.recv_string(0).unwrap().unwrap());
            assert_eq!(agent_sock.get_rcvmore().unwrap(), false);

            let fs = vec![FsMount {
                filesystem: "/dev/disk0".to_string(),
                mountpoint: "/".to_string(),
                size: 10000,
                used: 5000,
                available: 5000,
                capacity: 0.5,
                // inodes_used: 20,
                // inodes_available: 0,
                // inodes_capacity: 1.0,
            }];

            agent_sock.send_str("Ok", zmq::SNDMORE).unwrap();
            agent_sock.send_str(&json::encode(&fs).unwrap(), 0).unwrap();
        });

        let mut sock = ctx.socket(zmq::REQ).unwrap();
        sock.connect("inproc://test").unwrap();

        let mut host = Ffi__Host::from(Host::test_new(None, Some(sock), None, None));
        let mut telemetry = telemetry_new();
        let ffi_fs = telemetry_fs(&mut telemetry as *mut Ffi__Telemetry, &mut host as *mut Ffi__Host).unwrap();

        let mut fs_vec = unsafe { Vec::from_raw_parts(ffi_fs.ptr, ffi_fs.length, ffi_fs.capacity) };
        let fs: Vec<_> = fs_vec.drain(..).map(|mount| FsMount::from(mount)).collect();

        assert_eq!(fs.first().unwrap().filesystem, "/dev/disk0");

        Host::from(host);

        agent_mock.join().unwrap();
    }

    #[cfg(feature = "remote-run")]
    #[test]
    fn test_telemetry_hostname() {
        let mut ctx = zmq::Context::new();

        let mut agent_sock = ctx.socket(zmq::REP).unwrap();
        agent_sock.bind("inproc://test").unwrap();

        let agent_mock = thread::spawn(move || {
            assert_eq!("telemetry::hostname", agent_sock.recv_string(0).unwrap().unwrap());
            assert_eq!(agent_sock.get_rcvmore().unwrap(), false);

            agent_sock.send_str("Ok", zmq::SNDMORE).unwrap();
            agent_sock.send_str("example.com", 0).unwrap();
        });

        let mut sock = ctx.socket(zmq::REQ).unwrap();
        sock.connect("inproc://test").unwrap();

        let mut host = Ffi__Host::from(Host::test_new(None, Some(sock), None, None));
        let mut telemetry = telemetry_new();
        let hostname = telemetry_hostname(&mut telemetry as *mut Ffi__Telemetry, &mut host as *mut Ffi__Host);

        assert_eq!(unsafe { str::from_utf8(CStr::from_ptr(hostname).to_bytes()).unwrap() }, "example.com");
        assert_eq!(unsafe { str::from_utf8(CStr::from_ptr(telemetry.hostname.unwrap()).to_bytes()).unwrap() }, "example.com");

        Host::from(host);

        agent_mock.join().unwrap();
    }

    #[cfg(feature = "remote-run")]
    #[test]
    fn test_telemetry_memory() {
        let mut ctx = zmq::Context::new();

        let mut agent_sock = ctx.socket(zmq::REP).unwrap();
        agent_sock.bind("inproc://test").unwrap();

        let agent_mock = thread::spawn(move || {
            assert_eq!("telemetry::memory", agent_sock.recv_string(0).unwrap().unwrap());
            assert_eq!(agent_sock.get_rcvmore().unwrap(), false);

            agent_sock.send_str("Ok", zmq::SNDMORE).unwrap();
            agent_sock.send_str("10240", 0).unwrap();
        });

        let mut sock = ctx.socket(zmq::REQ).unwrap();
        sock.connect("inproc://test").unwrap();

        let mut host = Ffi__Host::from(Host::test_new(None, Some(sock), None, None));
        let mut telemetry = telemetry_new();
        let memory = telemetry_memory(&mut telemetry as *mut Ffi__Telemetry, &mut host as *mut Ffi__Host);

        assert_eq!(memory, 10240);
        assert_eq!(telemetry.memory.unwrap(), 10240);

        Host::from(host);

        agent_mock.join().unwrap();
    }

    #[cfg(feature = "remote-run")]
    #[test]
    fn test_telemetry_net() {
        let mut ctx = zmq::Context::new();

        let mut agent_sock = ctx.socket(zmq::REP).unwrap();
        agent_sock.bind("inproc://test").unwrap();

        let agent_mock = thread::spawn(move || {
            assert_eq!("telemetry::net", agent_sock.recv_string(0).unwrap().unwrap());
            assert_eq!(agent_sock.get_rcvmore().unwrap(), false);

            let net = vec![Netif {
                interface: "em0".to_string(),
                mac: Some("01:23:45:67:89:ab".to_string()),
                inet: Some(NetifIPv4 {
                    address: "127.0.0.1".to_string(),
                    netmask: "255.255.255.255".to_string(),
                }),
                inet6: Some(NetifIPv6 {
                    address: "::1".to_string(),
                    prefixlen: 8,
                    scopeid: Some("0x4".to_string()),
                }),
                status: Some(NetifStatus::Active),
            }];

            agent_sock.send_str("Ok", zmq::SNDMORE).unwrap();
            agent_sock.send_str(&json::encode(&net).unwrap(), 0).unwrap();
        });

        let mut sock = ctx.socket(zmq::REQ).unwrap();
        sock.connect("inproc://test").unwrap();

        let mut host = Ffi__Host::from(Host::test_new(None, Some(sock), None, None));
        let mut telemetry = telemetry_new();
        let ffi_net = telemetry_net(&mut telemetry as *mut Ffi__Telemetry, &mut host as *mut Ffi__Host).unwrap();

        let mut net_vec = unsafe { Vec::from_raw_parts(ffi_net.ptr, ffi_net.length, ffi_net.capacity) };
        let net: Vec<_> = net_vec.drain(..).map(|netif| Netif::from(netif)).collect();

        assert_eq!(net.first().unwrap().interface, "em0");

        Host::from(host);

        agent_mock.join().unwrap();
    }

    #[cfg(feature = "remote-run")]
    #[test]
    fn test_telemetry_os() {
        let mut ctx = zmq::Context::new();

        let mut agent_sock = ctx.socket(zmq::REP).unwrap();
        agent_sock.bind("inproc://test").unwrap();

        let agent_mock = thread::spawn(move || {
            assert_eq!("telemetry::os", agent_sock.recv_string(0).unwrap().unwrap());
            assert_eq!(agent_sock.get_rcvmore().unwrap(), false);

            let os = Os {
                arch: "doctor string".to_string(),
                family: "moo".to_string(),
                platform: OsPlatform::Centos,
                version: "1.0".to_string(),
            };

            agent_sock.send_str("Ok", zmq::SNDMORE).unwrap();
            agent_sock.send_str(&json::encode(&os).unwrap(), 0).unwrap();
        });

        let mut sock = ctx.socket(zmq::REQ).unwrap();
        sock.connect("inproc://test").unwrap();

        let mut host = Ffi__Host::from(Host::test_new(None, Some(sock), None, None));
        let mut telemetry = telemetry_new();
        let os = telemetry_os(&mut telemetry as *mut Ffi__Telemetry, &mut host as *mut Ffi__Host);

        assert_eq!(unsafe { str::from_utf8(CStr::from_ptr(os.unwrap().family).to_bytes()).unwrap() }, "moo");
        assert_eq!(unsafe { str::from_utf8(CStr::from_ptr(telemetry.os.unwrap().family).to_bytes()).unwrap() }, "moo");

        Host::from(host);

        agent_mock.join().unwrap();
    }

    fn create_telemetry() -> Telemetry {
        Telemetry {
            cpu: Some(Cpu {
                vendor: "moo".to_string(),
                brand_string: "Moo Cow Super Fun Happy CPU".to_string(),
                cores: 100,
            }),
            fs: Some(vec![FsMount {
                filesystem: "/dev/disk0".to_string(),
                mountpoint: "/".to_string(),
                size: 10000,
                used: 5000,
                available: 5000,
                capacity: 0.5,
                // inodes_used: 20,
                // inodes_available: 0,
                // inodes_capacity: 1.0,
            }]),
            hostname: Some("localhost".to_string()),
            memory: Some(2048),
            net: Some(vec![Netif {
                interface: "em0".to_string(),
                mac: Some("01:23:45:67:89:ab".to_string()),
                inet: Some(NetifIPv4 {
                    address: "127.0.0.1".to_string(),
                    netmask: "255.255.255.255".to_string(),
                }),
                inet6: Some(NetifIPv6 {
                    address: "::1".to_string(),
                    prefixlen: 8,
                    scopeid: Some("0x4".to_string()),
                }),
                status: Some(NetifStatus::Active),
            }]),
            os: Some(Os {
                arch: "doctor string".to_string(),
                family: "moo".to_string(),
                platform: OsPlatform::Centos,
                version: "1.0".to_string(),
            }),
        }
    }

    fn create_ffi_telemetry() -> Ffi__Telemetry {
        let mut fs = vec![Ffi__FsMount {
            filesystem: CString::new("/dev/disk0").unwrap().into_raw(),
            mountpoint: CString::new("/").unwrap().into_raw(),
            size: 10000 as uint64_t,
            used: 5000 as uint64_t,
            available: 5000 as uint64_t,
            capacity: 0.5 as c_float,
//            inodes_used: 20 as uint64_t,
//            inodes_available: 0 as uint64_t,
//            inodes_capacity: 1.0 as c_float,
        }];

        let mut net = vec![Ffi__Netif {
            interface: CString::new("em0").unwrap().into_raw(),
            mac: CString::new("01:23:45:67:89:ab").unwrap().into_raw(),
            inet: Ffi__NetifIPv4 {
                address: CString::new("01:23:45:67:89:ab").unwrap().into_raw(),
                netmask: CString::new("255.255.255.255").unwrap().into_raw(),
            },
            inet6: Ffi__NetifIPv6 {
                address: CString::new("::1").unwrap().into_raw(),
                prefixlen: 8 as uint32_t,
                scopeid: CString::new("0x4").unwrap().into_raw(),
            },
            status: CString::new("Active").unwrap().into_raw(),
        }];

        let ffi_telemetry = Ffi__Telemetry {
            cpu: Some(Ffi__Cpu {
                vendor: CString::new("moo").unwrap().into_raw(),
                brand_string: CString::new("Moo Cow Super Fun Happy CPU").unwrap().into_raw(),
                cores: 100 as uint32_t,
            }),
            fs: Some(Ffi__Array {
                ptr: fs.as_mut_ptr(),
                length: fs.len() as size_t,
                capacity: fs.capacity() as size_t,
            }),
            hostname: Some(CString::new("localhost").unwrap().into_raw()),
            memory: Some(1024),
            net: Some(Ffi__Array {
                ptr: net.as_mut_ptr(),
                length: net.len() as size_t,
                capacity: net.capacity() as size_t,
            }),
            os: Some(Ffi__Os {
                arch: CString::new("doctor string").unwrap().into_raw(),
                family: CString::new("moo").unwrap().into_raw(),
                platform: OsPlatform::Centos,
                version: CString::new("1.0").unwrap().into_raw(),
            }),
        };

        // Note: This causes a memory leak but unless we forget them,
        // Rust will deallocate the memory and Telemetry::from() will
        // segfault.
        mem::forget(fs);
        mem::forget(net);

        ffi_telemetry
    }
}
