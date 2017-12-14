// Copyright 2015-2017 Intecture Developers.
//
// Licensed under the Mozilla Public License 2.0 <LICENSE or
// https://www.tldrlegal.com/l/mpl-2.0>. This file may not be copied,
// modified, or distributed except according to those terms.

use command::Child;
use error_chain::ChainedError;
use errors::*;
use futures::{future, Future};
use futures::future::FutureResult;
use host::local::Local;
use std::process;
use super::{Launchctl, ServiceProvider};
use telemetry::Telemetry;

pub struct Homebrew {
    inner: Launchctl,
}

impl Homebrew {
    #[doc(hidden)]
    pub fn new(telemetry: &Telemetry) -> Homebrew {
        Homebrew {
            inner: Launchctl::new(telemetry),
        }
    }
}

impl ServiceProvider for Homebrew {
    fn available(telemetry: &Telemetry) -> Result<bool> {
        let brew = process::Command::new("/usr/bin/type")
            .arg("brew")
            .status()
            .chain_err(|| "Could not determine provider availability")?
            .success();

        Ok(brew && Launchctl::available(telemetry)?)
    }

    fn running(&self, host: &Local, name: &str) -> Box<Future<Item = bool, Error = Error>> {
        self.inner.running(host, name)
    }

    fn action(&self, host: &Local, name: &str, action: &str) -> FutureResult<Child, Error> {
        // @todo This isn't the most reliable method. Ideally a user would
        // invoke these commands themselves.
        let result = if action == "stop" {
            self.inner.uninstall_plist(name)
        } else {
            let path = format!("/usr/local/opt/{}/homebrew.mxcl.{0}.plist", name);
            self.inner.install_plist(path)
        };

        match result {
            Ok(_) => self.inner.action(host, name, action),
            Err(e) => future::err(format!("{}", e.display_chain()).into())
        }
    }

    fn enabled(&self, host: &Local, name: &str) -> Box<Future<Item = bool, Error = Error>> {
        self.inner.enabled(host, name)
    }

    fn enable(&self, host: &Local, name: &str) -> Box<Future<Item = (), Error = Error>> {
        self.inner.enable(host, name)
    }

    fn disable(&self, host: &Local, name: &str) -> Box<Future<Item = (), Error = Error>> {
        self.inner.disable(host, name)
    }
}
