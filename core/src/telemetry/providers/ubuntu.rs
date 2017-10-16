// Copyright 2015-2017 Intecture Developers.
//
// Licensed under the Mozilla Public License 2.0 <LICENSE or
// https://www.tldrlegal.com/l/mpl-2.0>. This file may not be copied,
// modified, or distributed except according to those terms.

use erased_serde::Serialize;
use errors::*;
use futures::{future, Future};
use host::{Host, HostType};
use host::local::Local;
use host::remote::Plain;
use pnet::datalink::interfaces;
use provider::Provider;
use regex::Regex;
use remote::{Executable, Runnable};
use std::{env, process, str};
use super::{TelemetryProvider, TelemetryRunnable};
use target::{default, linux};
use target::linux::LinuxFlavour;
use telemetry::{Cpu, Os, OsFamily, OsPlatform, Telemetry, serializable};

pub struct Ubuntu<H: Host> {
    host: H,
}

struct LocalUbuntu;
struct RemoteUbuntu;

#[doc(hidden)]
#[derive(Serialize, Deserialize)]
pub enum UbuntuRunnable {
    Available,
    Load,
}

impl<H: Host + 'static> Provider<H> for Ubuntu<H> {
    fn available(host: &H) -> Box<Future<Item = bool, Error = Error>> {
        match host.get_type() {
            HostType::Local(l) => LocalUbuntu::available(l),
            HostType::Remote(r) => RemoteUbuntu::available(r),
        }
    }

    fn try_new(host: &H) -> Box<Future<Item = Option<Ubuntu<H>>, Error = Error>> {
        let host = host.clone();
        Box::new(Self::available(&host)
            .and_then(|available| {
                if available {
                    future::ok(Some(Ubuntu { host }))
                } else {
                    future::ok(None)
                }
            }))
    }
}

impl<H: Host + 'static> TelemetryProvider<H> for Ubuntu<H> {
    fn load(&mut self) -> Box<Future<Item = Telemetry, Error = Error>> {
        match self.host.get_type() {
            HostType::Local(l) => LocalUbuntu::load(l),
            HostType::Remote(r) => RemoteUbuntu::load(r),
        }
    }
}

impl LocalUbuntu {
    fn available(_: &Local) -> Box<Future<Item = bool, Error = Error>> {
        Box::new(future::ok(cfg!(target_os="linux") && linux::fingerprint_os() == Some(LinuxFlavour::Ubuntu)))
    }

    fn load(_: &Local) -> Box<Future<Item = Telemetry, Error = Error>> {
        Box::new(future::lazy(|| match do_load() {
            Ok(t) => future::ok(t),
            Err(e) => future::err(e),
        }))
    }
}

impl RemoteUbuntu {
    fn available(host: &Plain) -> Box<Future<Item = bool, Error = Error>> {
        let runnable = Runnable::Telemetry(
                           TelemetryRunnable::Ubuntu(
                               UbuntuRunnable::Available));
        host.run(runnable)
            .chain_err(|| ErrorKind::Runnable { endpoint: "Telemetry::Ubuntu", func: "available" })
    }

    fn load(host: &Plain) -> Box<Future<Item = Telemetry, Error = Error>> {
        let runnable = Runnable::Telemetry(
                           TelemetryRunnable::Ubuntu(
                               UbuntuRunnable::Load));
        let host = host.clone();

        Box::new(host.run(runnable)
            .chain_err(|| ErrorKind::Runnable { endpoint: "Telemetry::Ubuntu", func: "load" })
            .map(|t: serializable::Telemetry| Telemetry::from(t)))
    }
}

impl Executable for UbuntuRunnable {
    fn exec(self, host: &Local) -> Box<Future<Item = Box<Serialize>, Error = Error>> {
        match self {
            UbuntuRunnable::Available => Box::new(LocalUbuntu::available(host).map(|b| Box::new(b) as Box<Serialize>)),
            UbuntuRunnable::Load => Box::new(LocalUbuntu::load(host).map(|t| {
                let t: serializable::Telemetry = t.into();
                Box::new(t) as Box<Serialize>
            }))
        }
    }
}

fn do_load() -> Result<Telemetry> {
    let (version_str, version_maj, version_min, version_patch) = version()?;

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
            family: OsFamily::Linux,
            platform: OsPlatform::Ubuntu,
            version_str: version_str,
            version_maj: version_maj,
            version_min: version_min,
            version_patch: version_patch,
        },
    })
}

fn version() -> Result<(String, u32, u32, u32)> {
    let out = process::Command::new("lsb_release").arg("-sd").output()?;
    let desc = str::from_utf8(&out.stdout)
                   .chain_err(|| ErrorKind::SystemCommand("Ubuntu-version"))?;

    let regex = Regex::new(r"([0-9]+)\.([0-9]+)\.([0-9]+)( LTS)?").unwrap();
    if let Some(cap) = regex.captures(&desc) {
        let version_maj = cap.get(1).unwrap().as_str().parse().chain_err(|| ErrorKind::SystemCommandOutput("lsb_release -sd"))?;
        let version_min = cap.get(2).unwrap().as_str().parse().chain_err(|| ErrorKind::SystemCommandOutput("lsb_release -sd"))?;
        let version_patch = cap.get(3).unwrap().as_str().parse().chain_err(|| ErrorKind::SystemCommandOutput("lsb_release -sd"))?;
        let mut version_str = format!("{}.{}.{}", version_maj, version_min, version_patch);
        if cap.get(4).is_some() {
            version_str.push_str(" LTS");
        }
        Ok((version_str, version_maj, version_min, version_patch))
    } else {
        Err(ErrorKind::SystemCommandOutput("lsb_release -sd").into())
    }
}
