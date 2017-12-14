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
use std::{fs, process};
use std::path::{Path, PathBuf};
use super::ServiceProvider;
use telemetry::{OsFamily, Telemetry};
use tokio_process::CommandExt;

pub struct Launchctl {
    domain_target: String,
    service_path: PathBuf,
}

impl Launchctl {
    #[doc(hidden)]
    pub fn new(telemetry: &Telemetry) -> Launchctl {
        let (domain_target, service_path) = if telemetry.user.is_root() {
            ("system".into(), "/Library/LaunchDaemons".into())
        } else {
            let mut path = telemetry.user.home_dir.clone();
            path.push("Library/LaunchAgents");
            (format!("gui/{}", telemetry.user.uid), path)
        };

        Launchctl { domain_target, service_path }
    }

    #[doc(hidden)]
    pub fn install_plist<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        if let Some(name) = path.as_ref().file_name() {
            let mut install_path = self.service_path.clone();

            // Create `Launch..` dir if it doesn't already exist.
            if !install_path.exists() {
                fs::create_dir(&install_path)?;
            }

            install_path.push(name);

            if !install_path.exists() {
                fs::copy(&path, &self.service_path)
                    .chain_err(|| "Could not install plist")?;
            }

            Ok(())
        } else {
            Err("Plist path does not contain filename".into())
        }
    }

    #[doc(hidden)]
    pub fn uninstall_plist(&self, name: &str) -> Result<()> {
        let mut path = self.service_path.clone();
        path.push(name);
        path.set_extension("plist");
        if path.exists() {
            fs::remove_file(&path)
                .chain_err(|| "Could not uninstall plist")?;
        }

        Ok(())
    }
}

impl ServiceProvider for Launchctl {
    fn available(telemetry: &Telemetry) -> Result<bool> {
        Ok(telemetry.os.family == OsFamily::Darwin && telemetry.os.version_min >= 11)
    }

    fn running(&self, host: &Local, name: &str) -> Box<Future<Item = bool, Error = Error>> {
        Box::new(match process::Command::new("/bin/launchctl")
            .args(&["blame", &format!("{}/{}", self.domain_target, name)])
            .status_async2(host.handle())
            .chain_err(|| "Error checking if service is running")
        {
            Ok(s) => s.map(|s| s.success())
                .map_err(|e| Error::with_chain(e, ErrorKind::SystemCommand("launchctl blame"))),
            Err(e) => return Box::new(future::err(e)),
        })
    }

    fn action(&self, host: &Local, name: &str, action: &str) -> FutureResult<Child, Error> {
        let action = match action {
            "start" => "bootstrap",
            "stop" => "bootout",
            "restart" => "kickstart -k",
            _ => action,
        };

        let cmd = match factory() {
            Ok(c) => c,
            Err(e) => return future::err(format!("{}", e.display_chain()).into()),
        };

        // Run through shell as `action` may contain multiple args with spaces.
        // If we passed `action` as a single argument, it would automatically
        // be quoted and multiple args would appear as a single quoted arg.
        cmd.exec(host, &[
            "/bin/sh",
            "-c",
            &format!("/bin/launchctl {} {} {}/{}.plist", action, self.domain_target, self.service_path.display(), name)
        ])
    }

    fn enabled(&self, host: &Local, name: &str) -> Box<Future<Item = bool, Error = Error>> {
        let name = name.to_owned();

        Box::new(process::Command::new("/bin/launchctl")
            .args(&["print-disabled", &self.domain_target])
            .output_async(host.handle())
            .map_err(|e| Error::with_chain(e, ErrorKind::SystemCommand("launchctl print-disabled <domain_target>")))
            .and_then(move |out| {
                if out.status.success() {
                    let re = match Regex::new(&format!("^\\s+\"{}\" => false", name)) {
                        Ok(r) => r,
                        Err(e) => return future::err(Error::with_chain(e, ErrorKind::Msg("Could not create Launchctl::enabled Regex".into())))
                    };
                    let stdout = String::from_utf8_lossy(&out.stdout);
                    let is_match = !re.is_match(&stdout);

                    future::ok(is_match)
                } else {
                    future::err(ErrorKind::SystemCommand("/bin/launchctl").into())
                }
            }))
    }

    fn enable(&self, host: &Local, name: &str) -> Box<Future<Item = (), Error = Error>> {
        Box::new(process::Command::new("/bin/launchctl")
            .args(&["enable", &format!("{}/{}", self.domain_target, name)])
            .output_async(host.handle())
            .map_err(|e| Error::with_chain(e, ErrorKind::SystemCommand("launchctl enable <service>")))
            .and_then(|out| {
                if out.status.success() {
                    future::ok(())
                } else {
                    future::err(format!("Could not enable service: {}", String::from_utf8_lossy(&out.stderr)).into())
                }
            }))
    }

    fn disable(&self, host: &Local, name: &str) -> Box<Future<Item = (), Error = Error>> {
        Box::new(process::Command::new("/bin/launchctl")
            .args(&["disable", &format!("{}/{}", self.domain_target, name)])
            .output_async(host.handle())
            .map_err(|e| Error::with_chain(e, ErrorKind::SystemCommand("launchctl disable <service>")))
            .and_then(|out| {
                if out.status.success() {
                    future::ok(())
                } else {
                    future::err(format!("Could not disable service: {}", String::from_utf8_lossy(&out.stderr)).into())
                }
            }))
    }
}
