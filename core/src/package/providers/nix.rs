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
use std::process;
use super::PackageProvider;
use tokio_process::CommandExt;

pub struct Nix;

impl PackageProvider for Nix {
    fn available() -> Result<bool> {
        Ok(process::Command::new("/usr/bin/type")
            .arg("nix-env")
            .status()
            .chain_err(|| "Could not determine provider availability")?
            .success())
    }

    fn installed(&self, host: &Local, name: &str) -> Box<Future<Item = bool, Error = Error>> {
        let name = name.to_owned();

        Box::new(process::Command::new("nix-env")
            .args(&["--install", "--dry-run", &name])
            .output_async(host.handle())
            .chain_err(|| "Could not check if package is installed")
            .and_then(move |output| {
                if output.status.success() {
                    let stdout = String::from_utf8_lossy(&output.stdout);
                    future::ok(!stdout.contains("these paths will be fetched"))
                } else {
                    future::err(format!("Error running `nix-env --install --dry-run {}`: {}",
                        name, String::from_utf8_lossy(&output.stderr)).into())
                }
            }))
    }

    fn install(&self, host: &Local, name: &str) -> FutureResult<Child, Error> {
        let cmd = match command::factory() {
            Ok(c) => c,
            Err(e) => return future::err(format!("{}", e.display_chain()).into()),
        };
        cmd.exec(host, &["nix-env", "--install", name])
    }

    fn uninstall(&self, host: &Local, name: &str) -> FutureResult<Child, Error> {
        let cmd = match command::factory() {
            Ok(c) => c,
            Err(e) => return future::err(format!("{}", e.display_chain()).into()),
        };
        cmd.exec(host, &["nix-env", "--uninstall", name])
    }
}
