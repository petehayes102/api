// Copyright 2015 Intecture Developers. See the COPYRIGHT file at the
// top-level directory of this distribution and at
// https://intecture.io/COPYRIGHT.
//
// Licensed under the Mozilla Public License 2.0 <LICENSE or
// https://www.tldrlegal.com/l/mpl-2.0>. This file may not be copied,
// modified, or distributed except according to those terms.

//! Data structures containing information about your managed host.

pub mod ffi;

use Result;
use std::convert::From;
use super::Host;
use target::Target;

#[derive(Debug, RustcDecodable, RustcEncodable)]
pub struct Telemetry {
    cpu: Option<Cpu>,
    fs: Option<Vec<FsMount>>,
    hostname: Option<String>,
    memory: Option<u64>,
    net: Option<Vec<Netif>>,
    os: Option<Os>,
}

impl Telemetry {
    pub fn new() -> Telemetry {
        Telemetry {
            cpu: None,
            fs: None,
            hostname: None,
            memory: None,
            net: None,
            os: None,
        }
    }

    pub fn get_cpu(&mut self, host: &mut Host) -> Result<&Cpu> {
        if self.cpu.is_none() {
            self.cpu = Some(try!(Target::telemetry_cpu(host)));
        }

        Ok(self.cpu.as_ref().unwrap())
    }

    pub fn get_fs(&mut self, host: &mut Host) -> Result<&Vec<FsMount>> {
        if self.fs.is_none() {
            self.fs = Some(try!(Target::telemetry_fs(host)));
        }

        Ok(self.fs.as_ref().unwrap())
    }

    pub fn get_hostname(&mut self, host: &mut Host) -> Result<&str> {
        if self.hostname.is_none() {
            self.hostname = Some(try!(Target::telemetry_hostname(host)));
        }

        Ok(self.hostname.as_ref().unwrap())
    }

    pub fn get_memory(&mut self, host: &mut Host) -> Result<u64> {
        if self.memory.is_none() {
            self.memory = Some(try!(Target::telemetry_memory(host)));
        }

        Ok(self.memory.unwrap())
    }

    pub fn get_net(&mut self, host: &mut Host) -> Result<&Vec<Netif>> {
        if self.net.is_none() {
            self.net = Some(try!(Target::telemetry_net(host)));
        }

        Ok(self.net.as_ref().unwrap())
    }

    pub fn get_os(&mut self, host: &mut Host) -> Result<&Os> {
        if self.os.is_none() {
            self.os = Some(try!(Target::telemetry_os(host)));
        }

        Ok(self.os.as_ref().unwrap())
    }
}

pub trait TelemetryTarget {
    fn telemetry_cpu(host: &mut Host) -> Result<Cpu>;
    fn telemetry_fs(host: &mut Host) -> Result<Vec<FsMount>>;
    fn telemetry_hostname(host: &mut Host) -> Result<String>;
    fn telemetry_memory(host: &mut Host) -> Result<u64>;
    fn telemetry_net(host: &mut Host) -> Result<Vec<Netif>>;
    fn telemetry_os(host: &mut Host) -> Result<Os>;
}

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

#[derive(Debug, PartialEq, RustcDecodable, RustcEncodable)]
#[repr(C)]
pub enum OsPlatform {
    Centos,
    Debian,
    Fedora,
    Freebsd,
    Macos,
    Redhat,
    Ubuntu,
}

impl From<String> for OsPlatform {
    fn from(platform: String) -> OsPlatform {
        match platform.as_ref() {
            "centos" => OsPlatform::Centos,
            "debian" => OsPlatform::Debian,
            "fedora" => OsPlatform::Fedora,
            "freebsd" => OsPlatform::Freebsd,
            "macos" => OsPlatform::Macos,
            "redhat" => OsPlatform::Redhat,
            "ubuntu" => OsPlatform::Ubuntu,
            _ => unimplemented!(),
        }
    }
}

#[derive(Debug, RustcDecodable, RustcEncodable)]
pub struct Os {
    pub arch: String,
    pub family: String,
    pub platform: OsPlatform,
    pub version: String,
}

impl Os {
    #[doc(hidden)]
    pub fn new(arch: &str, family: &str, platform: OsPlatform, version: &str) -> Os {
        Os {
            arch: arch.to_string(),
            family: family.to_string(),
            platform: platform,
            version: version.to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use Host;
    #[cfg(feature = "remote-run")]
    use rustc_serialize::json;
    #[cfg(feature = "remote-run")]
    use std::thread;
    use super::*;
    #[cfg(feature = "remote-run")]
    use zmq;

    // XXX Local tests require mocking

    #[cfg(feature = "remote-run")]
    #[test]
    fn test_get_cpu() {
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
        sock.set_linger(0).unwrap();
        sock.connect("inproc://test").unwrap();

        let mut host = Host::test_new(None, Some(sock), None, None);

        let mut telemetry = Telemetry::new();
        let cpu = telemetry.get_cpu(&mut host).unwrap();

        assert_eq!(&cpu.vendor, "moo");
        assert_eq!(cpu.cores, 100);

        agent_mock.join().unwrap();
    }

    #[cfg(feature = "remote-run")]
    #[test]
    fn test_get_fs() {
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
        sock.set_linger(0).unwrap();
        sock.connect("inproc://test").unwrap();

        let mut host = Host::test_new(None, Some(sock), None, None);

        let mut telemetry = Telemetry::new();
        let fs = telemetry.get_fs(&mut host).unwrap();

        assert_eq!(&fs.first().unwrap().mountpoint, "/");
        assert_eq!(fs.first().unwrap().capacity, 0.5);

        agent_mock.join().unwrap();
    }

    #[cfg(feature = "remote-run")]
    #[test]
    fn test_get_hostname() {
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
        sock.set_linger(0).unwrap();
        sock.connect("inproc://test").unwrap();

        let mut host = Host::test_new(None, Some(sock), None, None);

        let mut telemetry = Telemetry::new();
        assert_eq!(telemetry.get_hostname(&mut host).unwrap(), "example.com");

        agent_mock.join().unwrap();
    }

    #[cfg(feature = "remote-run")]
    #[test]
    fn test_get_memory() {
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
        sock.set_linger(0).unwrap();
        sock.connect("inproc://test").unwrap();

        let mut host = Host::test_new(None, Some(sock), None, None);

        let mut telemetry = Telemetry::new();
        assert_eq!(telemetry.get_memory(&mut host).unwrap(), 10240);

        agent_mock.join().unwrap();
    }

    #[cfg(feature = "remote-run")]
    #[test]
    fn test_get_net() {
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
        sock.set_linger(0).unwrap();
        sock.connect("inproc://test").unwrap();

        let mut host = Host::test_new(None, Some(sock), None, None);

        let mut telemetry = Telemetry::new();
        let net = telemetry.get_net(&mut host).unwrap();

        assert_eq!(&net.first().unwrap().interface, "em0");

        agent_mock.join().unwrap();
    }

    #[cfg(feature = "remote-run")]
    #[test]
    fn test_get_os() {
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
        sock.set_linger(0).unwrap();
        sock.connect("inproc://test").unwrap();

        let mut host = Host::test_new(None, Some(sock), None, None);

        let mut telemetry = Telemetry::new();
        let os = telemetry.get_os(&mut host).unwrap();

        assert_eq!(os.platform, OsPlatform::Centos);
        assert_eq!(&os.version, "1.0");

        agent_mock.join().unwrap();
    }
}
