// Copyright 2015-2017 Intecture Developers.
//
// Licensed under the Mozilla Public License 2.0 <LICENSE or
// https://www.tldrlegal.com/l/mpl-2.0>. This file may not be copied,
// modified, or distributed except according to those terms.

//! Endpoint for running shell commands.
//!
//! A shell command is represented by the `Command` struct, which is not
//! idempotent.

mod child;
pub mod providers;

pub use self::child::Child;

use errors::*;
use futures::Future;
use futures::future::FutureResult;
use host::Host;
use host::local::Local;
use request::Executable;

#[cfg(not(windows))]
const DEFAULT_SHELL: [&'static str; 2] = ["/bin/sh", "-c"];
#[cfg(windows)]
const DEFAULT_SHELL: [&'static str; 1] = ["yeah...we don't currently support windows :("];

/// Represents a shell command to be executed on a host.
///
///## Examples
///
/// Here's an example `ls` command that lists the directory `foo/`.
///
///```
///extern crate futures;
///extern crate intecture_api;
///extern crate tokio_core;
///
///use futures::{Future, Stream};
///use intecture_api::prelude::*;
///use tokio_core::reactor::Core;
///
///# fn main() {
///let mut core = Core::new().unwrap();
///let handle = core.handle();
///
///let host = Local::new(&handle).wait().unwrap();
///
///let cmd = Command::new(&host, "ls /path/to/foo", None);
///let result = cmd.exec().and_then(|mut status| {
///    // Print the command's stdout/stderr to stdout
///    status.take_stream().unwrap()
///        .for_each(|line| { println!("{}", line); Ok(()) })
///        // On its own, the stream will not do anything, so we need to make
///        // sure it gets returned along with the status future. `join()` will
///        // mash the two together so we can return them as one.
///        .join(status.map(|s| println!("This command {} {}",
///            if s.success { "succeeded" } else { "failed" },
///            if let Some(e) = s.code {
///                format!("with code {}", e)
///            } else {
///                String::new()
///            })))
///});
///
///core.run(result).unwrap();
///# }
///```
///
/// We can also save all output to a string for later use. **Be careful** doing
/// this as you could run out of memory on your heap if the output buffer is
/// too big.
///
///```no_run
///extern crate futures;
///extern crate intecture_api;
///extern crate tokio_core;
///
///use futures::Future;
///use intecture_api::errors::*;
///use intecture_api::prelude::*;
///use tokio_core::reactor::Core;
///
///# fn main() {
///let mut core = Core::new().unwrap();
///let handle = core.handle();
///
///let host = Local::new(&handle).wait().unwrap();
///
///let cmd = Command::new(&host, "ls /path/to/foo", None);
///let result = cmd.exec().and_then(|status| {
///    status.result().unwrap()
///        .map(|_output| {
///            // Our command finished successfully. Now we can do something
///            // with our output here.
///        })
///        .map_err(|e| {
///            // Our command errored out. Let's grab the output and see what
///            // went wrong.
///            match *e.kind() {
///                ErrorKind::Command(ref output) => println!("Oh noes! {}", output),
///                _ => unreachable!(),
///            }
///            e
///        })
///});
///
///core.run(result).unwrap();
///# }
///```
///
/// Finally, we can also ignore the stream entirely if we only care whether the
/// command succeeded or not.
///
///```
///extern crate futures;
///extern crate intecture_api;
///extern crate tokio_core;
///
///use futures::{Future, Stream};
///use intecture_api::prelude::*;
///use tokio_core::reactor::Core;
///
///# fn main() {
///let mut core = Core::new().unwrap();
///let handle = core.handle();
///
///let host = Local::new(&handle).wait().unwrap();
///
///let cmd = Command::new(&host, "ls /path/to/foo", None);
///let result = cmd.exec().and_then(|mut status| {
///    status.map(|exit_status| {
///        if exit_status.success {
///            println!("Huzzah!");
///        } else {
///            println!("Doh!");
///        }
///    })
///});
///
///core.run(result).unwrap();
///# }
///```
pub struct Command<H> {
    host: H,
    cmd: Vec<String>,
}

#[doc(hidden)]
#[derive(Serialize, Deserialize, FromMessage, IntoMessage)]
pub struct CommandExec {
    cmd: Vec<String>,
}

impl<H: Host + 'static> Command<H> {
    /// Create a new `Command` with the default [`Provider`](enum.Provider.html).
    ///
    /// By default, `Command` will use `/bin/sh -c` as the shell. You can
    /// override this by providing a value for `shell`. Note that the
    /// underlying implementation of `Command` escapes whitespace, so each
    /// argument needs to be a separate item in the slice. For example, to use
    /// Bash as your shell, you'd provide the value:
    /// `Some(&["/bin/bash", "-c"])`.
    pub fn new(host: &H, cmd: &str, shell: Option<&[&str]>) -> Self {
        let mut args: Vec<String> = shell.unwrap_or(&DEFAULT_SHELL).to_owned()
            .iter().map(|a| (*a).to_owned()).collect();
        args.push(cmd.into());

        Command {
            host: host.clone(),
            cmd: args,
        }
    }

    /// Execute the command.
    ///
    ///## Returns
    ///
    /// This function returns a `Future` that represents the delay between
    /// now and the time it takes to start execution. This `Future` yields a
    /// tuple with a `Stream` and a `Future` inside. The `Stream` is the
    /// command's output stream, including both stdout and stderr. The `Future`
    /// yields the command's `ExitStatus`.
    ///
    /// **WARNING!** For remote `Host` types, you _MUST_ consume the output
    /// `Stream` if you want to access the `ExitStatus`. This is due to the
    /// plumbing between the API and the remote host, which relies on a single
    /// streaming pipe. First we stream the command output, then tack the
    /// `ExitStatus` on as the last frame. Without consuming the output buffer,
    /// we would never be able to get to the last frame, and `ExitStatus` could
    /// never be resolved.
    ///
    ///# Errors
    ///
    ///>Error: Buffer dropped before ExitStatus was sent
    ///
    ///>Caused by: oneshot canceled
    ///
    /// This is the error you'll see if you prematurely drop the output `Stream`
    /// while trying to resolve the `Future<Item = ExitStatus, ...>`.
    pub fn exec(&self) -> Box<Future<Item = Child, Error = Error>> {
        Box::new(self.host.request(CommandExec { cmd: self.cmd.clone() })
            .chain_err(|| ErrorKind::Request { endpoint: "Command", func: "exec" }))
    }
}

impl Executable for CommandExec {
    type Response = Child;
    type Future = FutureResult<Self::Response, Error>;

    fn exec(self, host: &Local) -> Self::Future {
        let args: Vec<&str> = self.cmd.iter().map(|a| &**a).collect();
        host.command().exec(host, &args)
    }
}
