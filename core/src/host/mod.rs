// Copyright 2015-2017 Intecture Developers.
//
// Licensed under the Mozilla Public License 2.0 <LICENSE or
// https://www.tldrlegal.com/l/mpl-2.0>. This file may not be copied,
// modified, or distributed except according to those terms.

//! Manages the connection between the API and a server.

pub mod local;
pub mod remote;

use command;
use errors::*;
use futures::Future;
use message::IntoMessage;
use request::Executable;
use telemetry::Telemetry;
use tokio_core::reactor::Handle;

/// Trait for local and remote host types.
pub trait Host: Clone {
    /// Get `Telemetry` for this host.
    fn telemetry(&self) -> &Telemetry;

    /// Get `Handle` to Tokio reactor.
    fn handle(&self) -> &Handle;

    #[doc(hidden)]
    fn request<R>(&self, request: R) -> Box<Future<Item = R::Response, Error = Error>>
        where R: Executable + IntoMessage + 'static;

    /// Get a reference to the appropriate `Command` provider for this host.
    fn command(&self) -> &Box<command::providers::CommandProvider>;

    /// Override the default `Command` provider for this host.
    fn set_command<P: command::providers::CommandProvider + 'static>(&mut self, P) -> Result<()>;
}

struct Providers {
    command: Box<command::providers::CommandProvider>,
}

fn get_providers() -> Result<Providers> {
    Ok(Providers {
        command: command::providers::factory()?,
    })
}
