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
use std::fs::read_dir;
use std::process;
use super::ServiceProvider;
use telemetry::{LinuxDistro, OsFamily, Telemetry};
use tokio_process::CommandExt;

pub struct Debian;

impl ServiceProvider for Debian {
    fn available(telemetry: &Telemetry) -> Result<bool> {
        Ok(telemetry.os.family == OsFamily::Linux(LinuxDistro::Debian))
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

        Box::new(process::Command::new("/sbin/runlevel")
            .output_async(host.handle())
            .map_err(|e| Error::with_chain(e, ErrorKind::SystemCommand("/sbin/runlevel")))
            .and_then(move |output| {
                if output.status.success() {
                    let mut stdout = (*String::from_utf8_lossy(&output.stdout)).to_owned();
                    let runlevel = match stdout.pop() {
                        Some(c) => c,
                        None => return future::err("Could not determine current runlevel".into()),
                    };

                    let dir = match read_dir(&format!("/etc/rc{}.d", runlevel)) {
                        Ok(dir) => dir,
                        Err(e) => return future::err(Error::with_chain(e, ErrorKind::Msg("Could not read rc dir".into()))),
                    };

                    let regex = match Regex::new(&format!("/S[0-9]+{}$", name)) {
                        Ok(r) => r,
                        Err(e) => return future::err(Error::with_chain(e, ErrorKind::Msg("Could not create Debian::enabled regex".into()))),
                    };

                    let mut enabled = false;
                    for file in dir {
                        if let Ok(file) = file {
                            if regex.is_match(&file.file_name().to_string_lossy()) {
                                enabled = true;
                                break;
                            }
                        }
                    }

                    future::ok(enabled)
                } else {
                    future::err(ErrorKind::SystemCommand("/usr/bin/runlevel").into())
                }
            }))
    }

    fn enable(&self, host: &Local, name: &str) -> Box<Future<Item = (), Error = Error>> {
        Box::new(process::Command::new("/usr/sbin/update-rc.d")
            .args(&["enable", name])
            .output_async(host.handle())
            .map_err(|e| Error::with_chain(e, ErrorKind::SystemCommand("update-rc.d enable <service>")))
            .and_then(|out| {
                if out.status.success() {
                    future::ok(())
                } else {
                    future::err(format!("Could not enable service: {}", String::from_utf8_lossy(&out.stderr)).into())
                }
            }))
    }

    fn disable(&self, host: &Local, name: &str) -> Box<Future<Item = (), Error = Error>> {
        Box::new(process::Command::new("/usr/sbin/update-rc.d")
            .args(&["disable", name])
            .output_async(host.handle())
            .map_err(|e| Error::with_chain(e, ErrorKind::SystemCommand("update-rc.d disable <service>")))
            .and_then(|out| {
                if out.status.success() {
                    future::ok(())
                } else {
                    future::err(format!("Could not disable service: {}", String::from_utf8_lossy(&out.stderr)).into())
                }
            }))
    }
}
