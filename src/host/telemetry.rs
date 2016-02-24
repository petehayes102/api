// Copyright 2015 Intecture Developers. See the COPYRIGHT file at the
// top-level directory of this distribution and at
// https://intecture.io/COPYRIGHT.
//
// Licensed under the Mozilla Public License 2.0 <LICENSE or
// https://www.tldrlegal.com/l/mpl-2.0>. This file may not be copied,
// modified, or distributed except according to those terms.

//! Data structures containing information about your managed host.

use rustc_serialize;
use Targets;

#[derive(Debug, RustcDecodable, RustcEncodable)]
pub struct Cpu {
    pub vendor: String,
    pub brand_string: String,
    pub cores: u32,
}

impl Cpu {
    #[doc(hidden)]
    pub fn new(vendor: &str, brand_string: &str, cores: u32) -> Cpu {
        Cpu {
            vendor: vendor.to_string(),
            brand_string: brand_string.to_string(),
            cores: cores,
        }
    }
}

#[derive(Debug, RustcDecodable, RustcEncodable)]
pub struct FsMount {
    pub filesystem: String,
    pub mountpoint: String,
    pub size: u64,
    pub used: u64,
    pub available: u64,
    pub capacity: f32,
//    pub inodes_used: u64,
//    pub inodes_available: u64,
//    pub inodes_capacity: f32,
}

impl FsMount {
    #[doc(hidden)]
    pub fn new(filesystem: &str, mountpoint: &str, size: u64, used: u64, available: u64, capacity: f32/*, inodes_used: u64, inodes_available: u64, inodes_capacity: f32*/) -> FsMount {
        FsMount {
            filesystem: filesystem.to_string(),
            mountpoint: mountpoint.to_string(),
            size: size,
            used: used,
            available: available,
            capacity: capacity,
            // inodes_used: inodes_used,
            // inodes_available: inodes_available,
            // inodes_capacity: inodes_capacity,
        }
    }
}

#[derive(Debug, RustcDecodable, RustcEncodable)]
pub struct Netif {
    pub interface: String,
    pub mac: Option<String>,
    pub inet: Option<NetifIPv4>,
    pub inet6: Option<NetifIPv6>,
    pub status: Option<NetifStatus>,
}

impl Netif {
    #[doc(hidden)]
    pub fn new(interface: &str, mac: Option<&str>, inet: Option<NetifIPv4>, inet6: Option<NetifIPv6>, status: Option<NetifStatus>) -> Netif {
        Netif {
            interface: interface.to_string(),
            mac: if mac.is_some() {
                Some(mac.unwrap().to_string())
            } else {
                None
            },
            inet: inet,
            inet6: inet6,
            status: status,
        }
    }
}

#[derive(Debug, RustcDecodable, RustcEncodable)]
pub enum NetifStatus {
    Active,
    Inactive,
}

#[derive(Debug, RustcDecodable, RustcEncodable)]
pub struct NetifIPv4 {
    pub address: String,
    pub netmask: String,
}

impl NetifIPv4 {
    #[doc(hidden)]
    pub fn new(address: &str, netmask: &str) -> NetifIPv4 {
        NetifIPv4 {
            address: address.to_string(),
            netmask: netmask.to_string(),
        }
    }
}

#[derive(Debug, RustcDecodable, RustcEncodable)]
pub struct NetifIPv6 {
    pub address: String,
    pub prefixlen: u8,
    pub scopeid: Option<String>,
}

impl NetifIPv6 {
    #[doc(hidden)]
    pub fn new(address: &str, prefixlen: u8, scopeid: Option<&str>) -> NetifIPv6 {
        NetifIPv6 {
            address: address.to_string(),
            prefixlen: prefixlen,
            scopeid: if scopeid.is_some() {
                Some(scopeid.unwrap().to_string())
            } else {
                None
            }
        }
    }
}

#[derive(Debug, RustcDecodable, RustcEncodable)]
pub struct Os {
    pub arch: String,
    pub family: String,
    pub platform: String,
    pub target: Targets,
    pub version: String,
}

#[doc(hidden)]
impl Os {
    pub fn new(arch: &str, family: &str, platform: &str, version: &str) -> Os {
        Os {
            arch: arch.to_string(),
            family: family.to_string(),
            platform: platform.to_string(),
            version: version.to_string(),
        }
    }
}
