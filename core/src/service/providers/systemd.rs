// Copyright 2015-2017 Intecture Developers.
//
// Licensed under the Mozilla Public License 2.0 <LICENSE or
// https://www.tldrlegal.com/l/mpl-2.0>. This file may not be copied,
// modified, or distributed except according to those terms.

use command::{Child, factory};
use error_chain::ChainedError;
use errors::*;
use futures::{future, Future};
use futures::future::FutureResult;
use host::Host;
use host::local::Local;
use std::process;
use super::ServiceProvider;
use telemetry::Telemetry;
use tokio_process::CommandExt;

pub struct Systemd;

impl ServiceProvider for Systemd {
    fn available(_: &Telemetry) -> Result<bool> {
        let output = process::Command::new("/usr/bin/stat")
            .args(&["--format=%N", "/proc/1/exe"])
            .output()
            .chain_err(|| "Could not determine provider availability")?;

        if output.status.success() {
            let out = String::from_utf8_lossy(&output.stdout);
            Ok(out.contains("systemd"))
        } else {
            Err(ErrorKind::SystemCommand("/usr/bin/stat").into())
        }
    }

    fn running(&self, host: &Local, name: &str) -> Box<Future<Item = bool, Error = Error>> {
        Box::new(match process::Command::new("systemctl")
            .args(&["is-active", name])
            .status_async2(host.handle())
            .chain_err(|| "Error checking if service is running")
        {
            Ok(s) => s.map(|s| s.success())
                .map_err(|e| Error::with_chain(e, ErrorKind::SystemCommand("systemctl is-active"))),
            Err(e) => return Box::new(future::err(e)),
        })
    }

    fn action(&self, host: &Local, name: &str, action: &str) -> FutureResult<Child, Error> {
        let cmd = match factory() {
            Ok(c) => c,
            Err(e) => return future::err(format!("{}", e.display_chain()).into()),
        };
        cmd.exec(host, &["systemctl", action, name])
    }

    fn enabled(&self, host: &Local, name: &str) -> Box<Future<Item = bool, Error = Error>> {
        match process::Command::new("systemctl")
            .args(&["is-enabled", name])
            .status_async2(host.handle())
            .chain_err(|| "Error checking if service is enabled")
        {
            Ok(s) => Box::new(s.map(|s| s.success())
                .map_err(|e| Error::with_chain(e, ErrorKind::SystemCommand("systemctl is-enabled")))),
            Err(e) => Box::new(future::err(e)),
        }
    }

    fn enable(&self, host: &Local, name: &str) -> Box<Future<Item = (), Error = Error>> {
        Box::new(process::Command::new("systemctl")
            .args(&["enable", name])
            .output_async(host.handle())
            .map_err(|e| Error::with_chain(e, ErrorKind::SystemCommand("systemctl enable <service>")))
            .and_then(|out| {
                if out.status.success() {
                    future::ok(())
                } else {
                    future::err(format!("Could not enable service: {}", String::from_utf8_lossy(&out.stderr)).into())
                }
            }))
    }

    fn disable(&self, host: &Local, name: &str) -> Box<Future<Item = (), Error = Error>> {
        Box::new(process::Command::new("systemctl")
            .args(&["disable", name])
            .output_async(host.handle())
            .map_err(|e| Error::with_chain(e, ErrorKind::SystemCommand("systemctl disable <service>")))
            .and_then(|out| {
                if out.status.success() {
                    future::ok(())
                } else {
                    future::err(format!("Could not disable service: {}", String::from_utf8_lossy(&out.stderr)).into())
                }
            }))
    }
}
