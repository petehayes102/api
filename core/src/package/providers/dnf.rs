// Copyright 2015-2017 Intecture Developers.
//
// Licensed under the Mozilla Public License 2.0 <LICENSE or
// https://www.tldrlegal.com/l/mpl-2.0>. This file may not be copied,
// modified, or distributed except according to those terms.

use command::{self, Child};
use error_chain::ChainedError;
use errors::*;
use futures::{future, Future};
use futures::future::FutureResult;
use host::Host;
use host::local::Local;
use regex::Regex;
use std::process;
use super::PackageProvider;
use tokio_process::CommandExt;

pub struct Dnf;

impl PackageProvider for Dnf {
    fn available() -> Result<bool> {
        Ok(process::Command::new("/usr/bin/type")
            .arg("dnf")
            .status()
            .chain_err(|| "Could not determine provider availability")?
            .success())
    }

    fn installed(&self, host: &Local, name: &str) -> Box<Future<Item = bool, Error = Error>> {
        let name = name.to_owned();
        let arch = host.get_telemetry().os.arch.clone();

        Box::new(process::Command::new("dnf")
            .args(&["list", "installed"])
            .output_async(host.handle())
            .chain_err(|| "Could not get installed packages")
            .and_then(move |output| {
                if output.status.success() {
                    let re = match Regex::new(&format!("(?m)^{}\\.({}|noarch)\\s+", name, arch)) {
                        Ok(r) => r,
                        Err(e) => return future::err(ErrorKind::Regex(e).into()),
                    };
                    let stdout = String::from_utf8_lossy(&output.stdout);
                    future::ok(re.is_match(&stdout))
                } else {
                    future::err(format!("Error running `dnf list installed`: {}",
                        String::from_utf8_lossy(&output.stderr)).into())
                }
            }))
    }

    fn install(&self, host: &Local, name: &str) -> FutureResult<Child, Error> {
        let cmd = match command::providers::factory() {
            Ok(c) => c,
            Err(e) => return future::err(format!("{}", e.display_chain()).into()),
        };
        cmd.exec(host, &["dnf", "-y", "install", name])
    }

    fn uninstall(&self, host: &Local, name: &str) -> FutureResult<Child, Error> {
        let cmd = match command::providers::factory() {
            Ok(c) => c,
            Err(e) => return future::err(format!("{}", e.display_chain()).into()),
        };
        cmd.exec(host, &["dnf", "-y", "remove", name])
    }
}
