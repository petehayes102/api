// Copyright 2015-2017 Intecture Developers.
//
// Licensed under the Mozilla Public License 2.0 <LICENSE or
// https://www.tldrlegal.com/l/mpl-2.0>. This file may not be copied,
// modified, or distributed except according to those terms.

use errors::*;
use futures::{future, Future};
use host::Host;
use host::local::Local;
use message::{FromMessage, IntoMessage, InMessage};
use serde_json as json;
use tokio_core::reactor::Handle;
use tokio_proto::streaming::Message;

pub trait Executable {
    type Response: FromMessage + IntoMessage;
    type Future: Future<Item = Self::Response, Error = Error>;

    fn exec(self, &Local) -> Self::Future;
}

macro_rules! buildreq {
    ($( [ $m:ident, $i:ident ] ),+) => (
        #[derive(Serialize)]
        pub enum Request {
            $($i(::$m::$i)),+
        }

        #[derive(Deserialize)]
        pub enum RequestValues {
            $($i(json::Value)),+
        }

        impl Request {
            pub fn exec(self, host: &Local) -> Box<Future<Item = InMessage, Error = Error>> {
                let host = host.clone();

                match self {
                    $(Request::$i(req) => Box::new(req.exec(&host)
                            .and_then(move |res| match res.into_msg(host.handle()) {
                                Ok(m) => future::ok(m),
                                Err(e) => future::err(e),
                            }))),+
                }
            }
        }

        impl FromMessage for Request {
            fn from_msg(mut msg: InMessage) -> Result<Self> {
                let body = msg.take_body();
                let values: RequestValues = json::from_value(msg.into_inner())
                    .chain_err(|| "Could not deserialize Request")?;

                let request = match values {
                    $(RequestValues::$i(v) => Request::$i(::$m::$i::from_msg(match body {
                        Some(b) => Message::WithBody(v, b),
                        None => Message::WithoutBody(v),
                    })?)),+
                };

                Ok(request)
            }
        }

        impl IntoMessage for Request {
            fn into_msg(self, _: &Handle) -> Result<InMessage> {
                let value = json::to_value(self).chain_err(|| "Could not convert type into Message")?;
                Ok(Message::WithoutBody(value))
            }
        }
    );
}

buildreq!(
    [ command, CommandExec ],
    [ package, PackageInstalled ],
    [ package, PackageInstall ],
    [ package, PackageUninstall ],
    [ service, ServiceRunning ],
    [ service, ServiceAction ],
    [ service, ServiceEnabled ],
    [ service, ServiceEnable ],
    [ service, ServiceDisable ],
    [ telemetry, TelemetryLoad ]
);
