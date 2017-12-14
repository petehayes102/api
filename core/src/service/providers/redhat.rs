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
use telemetry::{LinuxDistro, OsFamily, Telemetry};
use tokio_process::CommandExt;

pub struct Redhat;

impl ServiceProvider for Redhat {
    fn available(telemetry: &Telemetry) -> Result<bool> {
        Ok(telemetry.os.family == OsFamily::Linux(LinuxDistro::RHEL))
    }

    fn running(&self, host: &Local, name: &str) -> Box<Future<Item = bool, Error = Error>> {
        Box::new(match process::Command::new("service")
            .args(&[name, "status"])
            .status_async2(host.handle())
            .chain_err(|| "Error checking if service is running")
        {
            Ok(s) => s.map(|s| s.success())
                .map_err(|e| Error::with_chain(e, ErrorKind::SystemCommand("service <service> status"))),
            Err(e) => return Box::new(future::err(e)),
        })
    }

    fn action(&self, host: &Local, name: &str, action: &str) -> FutureResult<Child, Error> {
        let cmd = match factory() {
            Ok(c) => c,
            Err(e) => return future::err(format!("{}", e.display_chain()).into()),
        };
        cmd.exec(host, &["service", action, name])
    }

    fn enabled(&self, host: &Local, name: &str) -> Box<Future<Item = bool, Error = Error>> {
        match process::Command::new("/usr/sbin/chkconfig")
            .arg(name)
            .status_async2(host.handle())
            .chain_err(|| "Error checking if service is enabled")
        {
            Ok(s) => Box::new(s.map(|s| s.success())
                .map_err(|e| Error::with_chain(e, ErrorKind::SystemCommand("chkconfig <service>")))),
            Err(e) => Box::new(future::err(e)),
        }
    }

    fn enable(&self, host: &Local, name: &str) -> Box<Future<Item = (), Error = Error>> {
        Box::new(process::Command::new("/usr/sbin/chkconfig")
            .args(&[name, "on"])
            .output_async(host.handle())
            .map_err(|e| Error::with_chain(e, ErrorKind::SystemCommand("chkconfig <service> on")))
            .and_then(|out| {
                if out.status.success() {
                    future::ok(())
                } else {
                    future::err(format!("Could not enable service: {}", String::from_utf8_lossy(&out.stderr)).into())
                }
            }))
    }

    fn disable(&self, host: &Local, name: &str) -> Box<Future<Item = (), Error = Error>> {
        Box::new(process::Command::new("/usr/sbin/chkconfig")
            .args(&[name, "off"])
            .output_async(host.handle())
            .map_err(|e| Error::with_chain(e, ErrorKind::SystemCommand("chkconfig <service> off")))
            .and_then(|out| {
                if out.status.success() {
                    future::ok(())
                } else {
                    future::err(format!("Could not disable service: {}", String::from_utf8_lossy(&out.stderr)).into())
                }
            }))
    }
}
