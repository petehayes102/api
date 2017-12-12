// Copyright 2015-2017 Intecture Developers.
//
// Licensed under the Mozilla Public License 2.0 <LICENSE or
// https://www.tldrlegal.com/l/mpl-2.0>. This file may not be copied,
// modified, or distributed except according to those terms.

//! OS abstractions for `Command`.

mod generic;

pub use self::generic::Generic;

use errors::*;
use futures::future::FutureResult;
use host::local::Local;
use super::Child;

#[doc(hidden)]
pub trait CommandProvider {
    fn available() -> bool where Self: Sized;
    fn exec(&self, &Local, &[&str]) -> FutureResult<Child, Error>;
}

#[doc(hidden)]
pub fn factory() -> Result<Box<CommandProvider>> {
    if Generic::available() {
        Ok(Box::new(Generic))
    } else {
        Err(ErrorKind::ProviderUnavailable("Command").into())
    }
}
