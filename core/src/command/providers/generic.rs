// Copyright 2015-2017 Intecture Developers.
//
// Licensed under the Mozilla Public License 2.0 <LICENSE or
// https://www.tldrlegal.com/l/mpl-2.0>. This file may not be copied,
// modified, or distributed except according to those terms.

use errors::*;
use futures::future::{self, FutureResult};
use host::Host;
use host::local::Local;
use std::process::{Command, Stdio};
use super::{Child, CommandProvider};
use tokio_process::CommandExt;

pub struct Generic;

impl CommandProvider for Generic {
    fn available() -> bool {
        true
    }

    fn exec(&self, host: &Local, cmd: &[&str]) -> FutureResult<Child, Error> {
        let result = cmd.split_first().ok_or("Invalid shell provided".into());
        let (cmd, cmd_args): (&&str, &[&str]) = match result {
            Ok((c, a)) => (c, a),
            Err(e) => return future::err(e),
        };

        match Command::new(cmd)
            .args(cmd_args)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn_async(host.handle())
            .chain_err(|| "Command execution failed")
        {
            Ok(child) => future::ok(child.into()),
            Err(e) => future::err(e),
        }
    }
}
