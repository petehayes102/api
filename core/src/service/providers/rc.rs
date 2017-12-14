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
use regex::Regex;
use std::process;
use super::ServiceProvider;
use telemetry::{OsFamily, Telemetry};
use tokio_process::CommandExt;

pub struct Rc;

impl ServiceProvider for Rc {
    fn available(telemetry: &Telemetry) -> Result<bool> {
        Ok(telemetry.os.family == OsFamily::Bsd)
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
        let name = name.to_owned();

        Box::new(process::Command::new("/usr/sbin/sysrc")
            .arg(&format!("{}_enable", name)) // XXX Assuming "_enable" is the correct suffix
            .output_async(host.handle())
            .map_err(|e| Error::with_chain(e, ErrorKind::SystemCommand("/usr/sbin/sysrc <service>_enable")))
            .and_then(move |output| {
                if output.status.success() {
                    let re = match Regex::new(&format!("^{}_enable: (?i:no)", name)) {
                        Ok(r) => r,
                        Err(e) => return future::err(Error::with_chain(e, ErrorKind::Msg("Could not create Rc::enabled Regex".into())))
                    };
                    let stdout = String::from_utf8_lossy(&output.stdout);

                    // XXX Assuming anything other than "no" is enabled
                    let is_match = !re.is_match(&stdout);

                    future::ok(is_match)
                } else {
                    future::ok(false)
                }
            }))
    }

    fn enable(&self, host: &Local, name: &str) -> Box<Future<Item = (), Error = Error>> {
        Box::new(process::Command::new("/usr/sbin/sysrc")
            .arg(&format!("{}_enable=\"YES\"", name))
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
        Box::new(process::Command::new("/usr/sbin/sysrc")
            .arg(&format!("{}_enable=\"NO\"", name))
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
