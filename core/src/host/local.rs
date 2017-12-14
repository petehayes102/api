// Copyright 2015-2017 Intecture Developers.
//
// Licensed under the Mozilla Public License 2.0 <LICENSE or
// https://www.tldrlegal.com/l/mpl-2.0>. This file may not be copied,
// modified, or distributed except according to those terms.

//! A connection to the local machine.

use command::CommandProvider;
use errors::*;
use futures::{future, Future};
use message::IntoMessage;
use package::PackageProvider;
use request::Executable;
use service::ServiceProvider;
use std::thread::sleep;
use std::time::Duration;
use std::sync::Arc;
use super::{Host, Providers};
use telemetry::{self, Telemetry};
use tokio_core::reactor::Handle;

/// A `Host` type that talks directly to the local machine.
#[derive(Clone)]
pub struct Local {
    inner: Arc<Inner>,
    handle: Handle,
}

struct Inner {
    providers: Option<Providers>,
    telemetry: Option<Telemetry>,
}

impl Local {
    /// Create a new `Host` targeting the local machine.
    pub fn new(handle: &Handle) -> Box<Future<Item = Self, Error = Error>> {
        let mut host = Local {
            inner: Arc::new(Inner {
                providers: None,
                telemetry: None,
            }),
            handle: handle.clone(),
        };

        Box::new(telemetry::Telemetry::load(&host)
            .chain_err(|| "Could not load telemetry for host")
            .and_then(|t| {
                {
                    let inner = Arc::get_mut(&mut host.inner).unwrap();
                    inner.providers = match super::get_providers(&t) {
                        Ok(p) => Some(p),
                        Err(e) => return future::err(e),
                    };
                    inner.telemetry = Some(t);
                }
                future::ok(host)
            }))
    }
}

impl Host for Local {
    fn telemetry(&self) -> &Telemetry {
        self.inner.telemetry.as_ref().unwrap()
    }

    fn handle(&self) -> &Handle {
        &self.handle
    }

    #[doc(hidden)]
    fn request<R>(&self, request: R) -> Box<Future<Item = R::Response, Error = Error>>
        where R: Executable + IntoMessage + 'static
    {
        Box::new(request.exec(self).and_then(|r| future::ok(r)))
    }

    fn command(&self) -> &Box<CommandProvider> {
        &self.inner.providers.as_ref().unwrap().command
    }

    fn set_command<P: CommandProvider + 'static>(&mut self, provider: P) -> Result<()> {
        // @todo Is this a good thing to do, or should we introduce a Mutex?
        for _ in 0..5 {
            match Arc::get_mut(&mut self.inner) {
                Some(inner) => {
                    inner.providers.as_mut().unwrap().command = Box::new(provider);
                    return Ok(());
                },
                None => sleep(Duration::from_millis(1)),
            }
        }

        Err(ErrorKind::MutRef("Local").into())
    }

    fn package(&self) -> &Box<PackageProvider> {
        &self.inner.providers.as_ref().unwrap().package
    }

    fn set_package<P: PackageProvider + 'static>(&mut self, provider: P) -> Result<()> {
        // @todo Is this a good thing to do, or should we introduce a Mutex?
        for _ in 0..5 {
            match Arc::get_mut(&mut self.inner) {
                Some(inner) => {
                    inner.providers.as_mut().unwrap().package = Box::new(provider);
                    return Ok(());
                },
                None => sleep(Duration::from_millis(1)),
            }
        }

        Err(ErrorKind::MutRef("Local").into())
    }

    fn service(&self) -> &Box<ServiceProvider> {
        &self.inner.providers.as_ref().unwrap().service
    }

    fn set_service<P: ServiceProvider + 'static>(&mut self, provider: P) -> Result<()> {
        // @todo Is this a good thing to do, or should we introduce a Mutex?
        for _ in 0..5 {
            match Arc::get_mut(&mut self.inner) {
                Some(inner) => {
                    inner.providers.as_mut().unwrap().service = Box::new(provider);
                    return Ok(());
                },
                None => sleep(Duration::from_millis(1)),
            }
        }

        Err(ErrorKind::MutRef("Local").into())
    }
}
