// Copyright 2015-2017 Intecture Developers.
//
// Licensed under the Mozilla Public License 2.0 <LICENSE or
// https://www.tldrlegal.com/l/mpl-2.0>. This file may not be copied,
// modified, or distributed except according to those terms.

use bytes::Bytes;
use errors::*;
use futures::{future, Future, Poll, Stream};
use futures::sink::Sink;
use futures::sync::{mpsc, oneshot};
use message::{FromMessage, IntoMessage, InMessage};
use serde_json as json;
use std::convert::From;
use std::io::{self, BufReader};
use std::result;
use tokio_core::reactor::Handle;
use tokio_io::io::lines;
use tokio_process;
use tokio_proto::streaming::{Body, Message};

/// Represents the status of a running `Command`, including the output stream
/// and exit status.
pub struct Child {
    exit_status: Option<Box<Future<Item = ExitStatus, Error = Error>>>,
    stream: Option<Box<Stream<Item = String, Error = Error>>>,
}

/// Represents the exit status of a `Command` as a `Result`-like `Future`. If
/// the command succeeded, the command output is returned. If it failed, an
/// error containing the command's output is returned.
pub struct CommandResult {
    inner: Box<Future<Item = String, Error = Error>>,
}

/// The status of a finished command.
///
/// This is a serializable replica of
/// [`std::process::ExitStatus`](https://doc.rust-lang.org/std/process/struct.ExitStatus.html).
#[derive(Debug, Serialize, Deserialize)]
pub struct ExitStatus {
    /// Was termination successful? Signal termination is not considered a
    /// success, and success is defined as a zero exit status.
    pub success: bool,
    /// Returns the exit code of the process, if any.
    ///
    /// On Unix, this will return `None` if the process was terminated by a
    /// signal.
    pub code: Option<i32>,
}

impl Child {
    /// Take ownership of the output stream.
    ///
    /// The stream is guaranteed to be present only if this is the first call
    /// to `take_stream()` and the future has not yet been polled.
    pub fn take_stream(&mut self) -> Option<Box<Stream<Item = String, Error = Error>>> {
        self.stream.take()
    }

    /// Convert this to a `CommandResult`, which returns the output string on
    /// success and an error containing the command's output on failure. If the
    /// stream has already been taken by `take_stream()` then this function
    /// will return `None`.
    ///
    /// Note that "success" is determined by examining the `ExitStatus::success`
    /// bool. See `ExitStatus` docs for details.
    pub fn result(self) -> Option<CommandResult> {
        if let Some(stream) = self.stream {
            let inner = stream.fold(String::new(), |mut acc, line| {
                    acc.push_str(&line);
                    future::ok::<_, Error>(acc)
                })
                .join(self.exit_status.unwrap())
                .and_then(|(output, status)| if status.success {
                    future::ok(output)
                } else {
                    future::err(ErrorKind::Command(output).into())
                });

            Some(CommandResult {
                inner: Box::new(inner) as Box<Future<Item = String, Error = Error>>
            })
        } else {
            None
        }
    }
}

impl From<tokio_process::Child> for Child {
    fn from(mut child: tokio_process::Child) -> Self {
        let stdout = child.stdout().take().expect("Child was not configured with stdout");
        let outbuf = BufReader::new(stdout);
        let stderr = child.stderr().take().expect("Child was not configured with stderr");
        let errbuf = BufReader::new(stderr);

        let stream = lines(outbuf)
            .select(lines(errbuf))
            .map_err(|e| Error::with_chain(e, ErrorKind::Msg("Command execution failed".into())));

        let status = child.map(|s| {
                ExitStatus {
                    success: s.success(),
                    code: s.code(),
                }
            })
            .map_err(|e| Error::with_chain(e, ErrorKind::Msg("Command execution failed".into())));

        Child {
            exit_status: Some(Box::new(status)),
            stream: Some(Box::new(stream)),
        }
    }
}

impl Future for Child {
    type Item = ExitStatus;
    type Error = Error;

    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        if let Some(stream) = self.stream.take() {
            self.exit_status = Some(Box::new(stream.for_each(|_| Ok(()))
                .join(self.exit_status.take().unwrap())
                .map(|(_, status)| status)));
        }

        self.exit_status.as_mut().unwrap().poll()
    }
}

impl FromMessage for Child {
    fn from_msg(mut msg: InMessage) -> Result<Self> {
        let (tx, rx) = oneshot::channel::<ExitStatus>();
        let mut tx = Some(tx);
        let stream = msg.take_body()
            .expect("Command::exec reply missing body stream")
            .filter_map(move |v| {
                let s = String::from_utf8_lossy(&v).to_string();

                // @todo This is a heuristical approach which is fallible
                if s.starts_with("ExitStatus:") {
                    let (_, json) = s.split_at(11);
                    match json::from_str(json) {
                        Ok(status) => {
                            // @todo What should happen if this fails?
                            let _ = tx.take().unwrap().send(status);
                            return None;
                        },
                        _ => (),
                    }
                }

                Some(s)
            })
            .then(|r| r.chain_err(|| "Command execution failed"));

        Ok(Child {
            exit_status: Some(Box::new(rx.chain_err(|| "Stream dropped before ExitStatus was sent"))),
            stream: Some(Box::new(stream)),
        })
    }
}

impl IntoMessage for Child {
    fn into_msg(self, handle: &Handle) -> Result<InMessage> {
        let (tx1, body) = Body::pair();
        let tx2 = tx1.clone();

        let status = self.exit_status.unwrap().and_then(|s| {
            match json::to_string(&s)
                .chain_err(|| "Could not serialize `ExitStatus` struct")
            {
                Ok(s) => {
                    let mut frame = "ExitStatus:".to_owned();
                    frame.push_str(&s);
                    Box::new(tx2.send(Ok(Bytes::from(frame.into_bytes())))
                        .map_err(|e| Error::with_chain(e, "Could not forward command output to Body"))
                    ) as Box<Future<Item = mpsc::Sender<result::Result<Bytes, io::Error>>, Error = Error>>
                },
                Err(e) => Box::new(future::err(e)),
            }
        });

        let stream = self.stream.unwrap().map(|s| Ok(Bytes::from(s.into_bytes())))
            .forward(tx1.sink_map_err(|e| Error::with_chain(e, "Could not forward command output to Body")))
            .join(status)
            // @todo We should repatriate these errors somehow
            .map(|_| ())
            .map_err(|_| ());

        handle.spawn(stream);

        let value: result::Result<_, ()> = Ok(());
        Ok(Message::WithBody(json::to_value(value).unwrap(), body))
    }
}

impl Future for CommandResult {
    type Item = String;
    type Error = Error;

    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        self.inner.poll()
    }
}
