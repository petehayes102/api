// Copyright 2015-2017 Intecture Developers.
//
// Licensed under the Mozilla Public License 2.0 <LICENSE or
// https://www.tldrlegal.com/l/mpl-2.0>. This file may not be copied,
// modified, or distributed except according to those terms.

//! Endpoint for managing system services.
//!
//! A service is represented by the `Service` struct, which is idempotent. This
//! means you can execute it repeatedly and it'll only run as needed.

mod providers;

use command::Child;
use errors::*;
use futures::{future, Future};
use futures::future::FutureResult;
use host::Host;
#[doc(hidden)]
pub use self::providers::{
    factory, ServiceProvider, Debian, Homebrew, Launchctl,
    Rc, Redhat, Systemd
};
pub use self::providers::Provider;

/// Represents a system service to be managed for a host.
///
///## Example
///
/// Enable and start a service.
///
///```no_run
///extern crate futures;
///extern crate intecture_api;
///extern crate tokio_core;
///
///use futures::{future, Future};
///use intecture_api::errors::Error;
///use intecture_api::prelude::*;
///use tokio_core::reactor::Core;
///
///# fn main() {
///let mut core = Core::new().unwrap();
///let handle = core.handle();
///
///let host = Local::new(&handle).wait().unwrap();
///
///let nginx = Service::new(&host, "nginx");
///let result = nginx.enable()
///    .and_then(|_| {
///        nginx.action("start")
///            .and_then(|maybe_status| {
///                match maybe_status {
///                    Some(status) => Box::new(status.result().unwrap().map(|_| ())) as Box<Future<Item = (), Error = Error>>,
///                    None => Box::new(future::ok(())),
///                }
///            })
///    });
///
///core.run(result).unwrap();
///# }
///```
pub struct Service<H: Host> {
    host: H,
    name: String,
}

#[doc(hidden)]
#[derive(Serialize, Deserialize, FromMessage, IntoMessage, Executable)]
#[response = "bool"]
#[hostarg = "true"]
pub struct ServiceRunning {
    name: String,
}

#[doc(hidden)]
#[derive(Serialize, Deserialize, FromMessage, IntoMessage, Executable)]
#[response = "Child"]
#[future = "FutureResult<Self::Response, Error>"]
#[hostarg = "true"]
pub struct ServiceAction {
    name: String,
    action: String,
}

#[doc(hidden)]
#[derive(Serialize, Deserialize, FromMessage, IntoMessage, Executable)]
#[response = "bool"]
#[hostarg = "true"]
pub struct ServiceEnabled {
    name: String,
}

#[doc(hidden)]
#[derive(Serialize, Deserialize, FromMessage, IntoMessage, Executable)]
#[response = "()"]
#[hostarg = "true"]
pub struct ServiceEnable {
    name: String,
}

#[doc(hidden)]
#[derive(Serialize, Deserialize, FromMessage, IntoMessage, Executable)]
#[response = "()"]
#[hostarg = "true"]
pub struct ServiceDisable {
    name: String,
}

impl<H: Host + 'static> Service<H> {
    /// Create a new `Service` with the default [`Provider`](enum.Provider.html).
    pub fn new(host: &H, name: &str) -> Service<H> {
        Service {
            host: host.clone(),
            name: name.into(),
        }
    }

    /// Check if the service is currently running.
    pub fn running(&self) -> Box<Future<Item = bool, Error = Error>> {
        Box::new(self.host.request(ServiceRunning { name: self.name.clone() })
            .chain_err(|| ErrorKind::Request { endpoint: "Service", func: "running" }))
    }

    /// Perform an action for the service, e.g. "start".
    ///
    ///## Cross-platform services
    ///
    /// By design, actions are specific to a particular service and are not
    /// cross-platform. Actions are defined by the package maintainer that
    /// wrote the service configuration, thus users should take care that they
    /// adhere to the configuration for each platform they target.
    ///
    ///## Idempotence
    ///
    /// This function is idempotent when running either the "start" or "stop"
    /// actions, as it will check first whether the service is already running.
    /// Idempotence is represented by the type `Future<Item = Option<..>, ...>`.
    /// Thus if it returns `Option::None` then the service is already in the
    /// required state, and if it returns `Option::Some` then Intecture is
    /// attempting to transition the service to the required state.
    ///
    /// If this fn returns `Option::Some<..>`, the nested tuple will hold
    /// handles to the live output and the result of the action. Under the hood
    /// this reuses the `Command` endpoint, so see
    /// [`Command` docs](../command/struct.Command.html) for detailed
    /// usage.
    pub fn action(&self, action: &str) -> Box<Future<Item = Option<Child>, Error = Error>> {
        if action == "start" || action == "stop" {
            let host = self.host.clone();
            let name = self.name.clone();
            let action = action.to_owned();

            Box::new(self.running()
                .and_then(move |running| {
                    if (running && action == "start") || (!running && action == "stop") {
                        Box::new(future::ok(None)) as Box<Future<Item = _, Error = Error>>
                    } else {
                        Box::new(Self::do_action(&host, &name, &action)
                            .map(|c| Some(c)))
                    }
                }))
        } else {
            Box::new(Self::do_action(&self.host, &self.name, action)
                .map(|c| Some(c)))
        }
    }

    fn do_action(host: &H, name: &str, action: &str) -> Box<Future<Item = Child, Error = Error>> {
        Box::new(host.request(ServiceAction { name: name.into(), action: action.into() })
            .chain_err(|| ErrorKind::Request { endpoint: "Service", func: "action" }))
    }

    /// Check if the service will start at boot.
    pub fn enabled(&self) -> Box<Future<Item = bool, Error = Error>> {
        Box::new(self.host.request(ServiceEnabled { name: self.name.clone() })
            .chain_err(|| ErrorKind::Request { endpoint: "Service", func: "enabled" }))
    }

    /// Instruct the service to start at boot.
    ///
    ///## Idempotence
    ///
    /// This function is idempotent, which is represented by the type
    /// `Future<Item = Option<..>, ...>`. Thus if it returns `Option::None`
    /// then the service is already enabled, and if it returns `Option::Some`
    /// then Intecture is attempting to enable the service.
    ///
    /// If this fn returns `Option::Some<..>`, the nested tuple will hold
    /// handles to the live output and the 'enable' command result. Under
    /// the hood this reuses the `Command` endpoint, so see
    /// [`Command` docs](../command/struct.Command.html) for detailed
    /// usage.
    pub fn enable(&self) -> Box<Future<Item = Option<()>, Error = Error>>
    {
        let host = self.host.clone();
        let name = self.name.clone();

        Box::new(self.enabled()
            .and_then(move |enabled| {
                if enabled {
                    Box::new(future::ok(None)) as Box<Future<Item = _, Error = Error>>
                } else {
                    Box::new(host.request(ServiceEnable { name: name.into() })
                        .chain_err(|| ErrorKind::Request { endpoint: "Service", func: "enable" })
                        .map(|_| Some(())))
                }
            }))
    }

    /// Prevent the service from starting at boot.
    ///
    ///## Idempotence
    ///
    /// This function is idempotent, which is represented by the type
    /// `Future<Item = Option<..>, ...>`. Thus if it returns `Option::None`
    /// then the service is already disabled, and if it returns `Option::Some`
    /// then Intecture is attempting to disable the service.
    ///
    /// If this fn returns `Option::Some<..>`, the nested tuple will hold
    /// handles to the live output and the 'disable' command result. Under
    /// the hood this reuses the `Command` endpoint, so see
    /// [`Command` docs](../command/struct.Command.html) for detailed
    /// usage.
    pub fn disable(&self) -> Box<Future<Item = Option<()>, Error = Error>>
    {
        let host = self.host.clone();
        let name = self.name.clone();

        Box::new(self.enabled()
            .and_then(move |enabled| {
                if enabled {
                    Box::new(host.request(ServiceDisable { name: name.into() })
                        .chain_err(|| ErrorKind::Request { endpoint: "Service", func: "disable" })
                        .map(|_| Some(())))
                } else {
                    Box::new(future::ok(None)) as Box<Future<Item = _, Error = Error>>
                }
            }))
    }
}
