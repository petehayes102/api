// Copyright 2015-2017 Intecture Developers.
//
// Licensed under the Mozilla Public License 2.0 <LICENSE or
// https://www.tldrlegal.com/l/mpl-2.0>. This file may not be copied,
// modified, or distributed except according to those terms.

use errors::*;
use futures::{future, Future};
use pnet::datalink::interfaces;
use std::env;
use super::TelemetryProvider;
use target::{default, linux, redhat};
use target::linux::LinuxFlavour;
use telemetry::{Cpu, LinuxDistro, Os, OsFamily, OsPlatform, Telemetry};

pub struct Centos;

impl TelemetryProvider for Centos {
    fn available() -> bool {
        cfg!(target_os="linux") && linux::fingerprint_os() == Some(LinuxFlavour::Centos)
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
    let (version_str, version_maj, version_min, version_patch) = redhat::version()?;

    Ok(Telemetry {
        cpu: Cpu {
            vendor: linux::cpu_vendor()?,
            brand_string: linux::cpu_brand_string()?,
            cores: linux::cpu_cores()?,
        },
        fs: default::fs().chain_err(|| "could not resolve telemetry data")?,
        hostname: default::hostname()?,
        memory: linux::memory().chain_err(|| "could not resolve telemetry data")?,
        net: interfaces(),
        os: Os {
            arch: env::consts::ARCH.into(),
            family: OsFamily::Linux(LinuxDistro::RHEL),
            platform: OsPlatform::Centos,
            version_str: version_str,
            version_maj: version_maj,
            version_min: version_min,
            version_patch: version_patch,
        },
        user: default::user()?,
    })
}
