// Copyright 2015-2017 Intecture Developers.
//
// Licensed under the Mozilla Public License 2.0 <LICENSE or
// https://www.tldrlegal.com/l/mpl-2.0>. This file may not be copied,
// modified, or distributed except according to those terms.

//! A connection to the local machine.

use command::providers::CommandProvider;
use errors::*;
use futures::{future, Future};
use message::IntoMessage;
use request::Executable;
use std::thread::sleep;
use std::time::Duration;
use std::sync::Arc;
use super::{Host, Providers};
// use telemetry::{self, Telemetry};
use tokio_core::reactor::Handle;

/// A `Host` type that talks directly to the local machine.
#[derive(Clone)]
pub struct Local {
    inner: Arc<Inner>,
    handle: Handle,
}

struct Inner {
    providers: Providers,
    // telemetry: Option<Telemetry>,
}

impl Local {
    /// Create a new `Host` targeting the local machine.
    pub fn new(handle: &Handle) -> Box<Future<Item = Self, Error = Error>> {
        let providers = match super::get_providers() {
            Ok(p) => p,
            Err(e) => return Box::new(future::err(e)),
        };

        let mut host = Local {
            inner: Arc::new(Inner {
                providers: providers,
                // telemetry: None,
            }),
            handle: handle.clone(),
        };

        Box::new(future::ok(host))

        // Box::new(telemetry::Telemetry::load(&host)
        //     .chain_err(|| "Could not load telemetry for host")
        //     .map(|t| {
        //         Arc::get_mut(&mut host.inner).unwrap().telemetry = Some(t);
        //         host
        //     }))
    }
}

impl Host for Local {
    // fn telemetry(&self) -> &Telemetry {
    //     self.inner.telemetry.as_ref().unwrap()
    // }

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
        &self.inner.providers.command
    }

    fn set_command<P: CommandProvider + 'static>(&mut self, provider: P) -> Result<()> {
        // @todo Is this a good thing to do, or should we introduce a Mutex?
        for _ in 0..5 {
            match Arc::get_mut(&mut self.inner) {
                Some(inner) => {
                    inner.providers.command = Box::new(provider);
                    return Ok(());
                },
                None => sleep(Duration::from_millis(1)),
            }
        }

        Err(ErrorKind::MutRef("Local").into())
    }
}
