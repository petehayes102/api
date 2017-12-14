// Copyright 2015-2017 Intecture Developers.
//
// Licensed under the Mozilla Public License 2.0 <LICENSE or
// https://www.tldrlegal.com/l/mpl-2.0>. This file may not be copied,
// modified, or distributed except according to those terms.

//! OS abstractions for `Service`.

mod debian;
mod homebrew;
mod launchctl;
mod rc;
mod redhat;
mod systemd;

use command::Child;
use errors::*;
use futures::Future;
use futures::future::FutureResult;
use host::local::Local;
pub use self::debian::Debian;
pub use self::homebrew::Homebrew;
pub use self::launchctl::Launchctl;
pub use self::rc::Rc;
pub use self::redhat::Redhat;
pub use self::systemd::Systemd;
use telemetry::Telemetry;

/// Specific implementation of `Service`
#[derive(Clone, Copy, Serialize, Deserialize)]
pub enum Provider {
    Debian,
    Homebrew,
    Launchctl,
    Rc,
    Redhat,
    Systemd,
}

pub trait ServiceProvider {
    fn available(&Telemetry) -> Result<bool> where Self: Sized;
    fn running(&self, &Local, &str) -> Box<Future<Item = bool, Error = Error>>;
    fn action(&self, &Local, &str, &str) -> FutureResult<Child, Error>;
    fn enabled(&self, &Local, &str) -> Box<Future<Item = bool, Error = Error>>;
    fn enable(&self, &Local, &str) -> Box<Future<Item = (), Error = Error>>;
    fn disable(&self, &Local, &str) -> Box<Future<Item = (), Error = Error>>;
}

#[doc(hidden)]
pub fn factory(telemetry: &Telemetry) -> Result<Box<ServiceProvider>> {
    if Systemd::available(telemetry)? {
        Ok(Box::new(Systemd))
    } else if Debian::available(telemetry)? {
        Ok(Box::new(Debian))
    } else if Homebrew::available(telemetry)? {
        Ok(Box::new(Homebrew::new(telemetry)))
    } else if Launchctl::available(telemetry)? {
        Ok(Box::new(Launchctl::new(telemetry)))
    } else if Rc::available(telemetry)? {
        Ok(Box::new(Rc))
    } else if Redhat::available(telemetry)? {
        Ok(Box::new(Redhat))
    } else {
        Err(ErrorKind::ProviderUnavailable("Service").into())
    }
}
