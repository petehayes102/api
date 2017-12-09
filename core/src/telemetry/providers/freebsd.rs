// Copyright 2015-2017 Intecture Developers.
//
// Licensed under the Mozilla Public License 2.0 <LICENSE or
// https://www.tldrlegal.com/l/mpl-2.0>. This file may not be copied,
// modified, or distributed except according to those terms.

use errors::*;
use futures::{future, Future};
use pnet::datalink::interfaces;
use regex::Regex;
use std::{env, fs};
use std::io::Read;
use super::TelemetryProvider;
use target::{default, unix};
use telemetry::{Cpu, Os, OsFamily, OsPlatform, Telemetry};

pub struct Freebsd;

impl TelemetryProvider for Freebsd {
    fn available() -> bool {
        cfg!(target_os="freebsd")
    }

    fn load(&self) -> Box<Future<Item = Telemetry, Error = Error>> {
        Box::new(future::lazy(|| {
            let t = match do_load() {
                Ok(t) => t,
                Err(e) => return future::err(e),
            };

            future::ok(t.into())
        }))
    }
}

fn do_load() -> Result<Telemetry> {
    let (version_str, version_maj, version_min) = unix::version()?;

    Ok(Telemetry {
        cpu: Cpu {
            vendor: telemetry_cpu_vendor()?,
            brand_string: unix::get_sysctl_item("hw\\.model")?,
            cores: unix::get_sysctl_item("hw\\.ncpu")
                        .chain_err(|| "could not resolve telemetry data")?
                        .parse::<u32>()
                        .chain_err(|| "could not resolve telemetry data")?,
        },
        fs: default::fs()?,
        hostname: default::hostname()?,
        memory: unix::get_sysctl_item("hw\\.physmem")
                     .chain_err(|| "could not resolve telemetry data")?
                     .parse::<u64>()
                     .chain_err(|| "could not resolve telemetry data")?,
        net: interfaces(),
        os: Os {
            arch: env::consts::ARCH.into(),
            family: OsFamily::Bsd,
            platform: OsPlatform::Freebsd,
            version_str: version_str,
            version_maj: version_maj,
            version_min: version_min,
            version_patch: 0
        },
        user: default::user()?,
    })
}

fn telemetry_cpu_vendor() -> Result<String> {
    let mut fh = fs::File::open("/var/run/dmesg.boot")
                          .chain_err(|| ErrorKind::SystemFile("/var/run/dmesg.boot"))?;
    let mut fc = String::new();
    fh.read_to_string(&mut fc).chain_err(|| ErrorKind::SystemFileOutput("/var/run/dmesg.boot"))?;

    let regex = Regex::new(r#"(?m)^CPU:.+$\n\s+Origin="([A-Za-z]+)""#).unwrap();
    if let Some(cap) = regex.captures(&fc) {
        Ok(cap.get(1).unwrap().as_str().into())
    } else {
        Err(ErrorKind::SystemFileOutput("/var/run/dmesg.boot").into())
    }
}
