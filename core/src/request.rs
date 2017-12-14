// Copyright 2015-2017 Intecture Developers.
//
// Licensed under the Mozilla Public License 2.0 <LICENSE or
// https://www.tldrlegal.com/l/mpl-2.0>. This file may not be copied,
// modified, or distributed except according to those terms.

use command;
use errors::*;
use futures::{future, Future};
use host::Host;
use host::local::Local;
use message::{FromMessage, IntoMessage, InMessage};
use package;
use serde_json as json;
use service;
use telemetry;
use tokio_core::reactor::Handle;
use tokio_proto::streaming::Message;

macro_rules! partstomsg {
    ($v:expr, $b:expr) => (match $b {
        ::std::option::Option::Some(body) => ::tokio_proto::streaming::Message::WithBody($v, body),
        ::std::option::Option::None => ::tokio_proto::streaming::Message::WithoutBody($v),
    });
}

macro_rules! reqexec {
    ($self:ident, $host:ident, $($variant:ident),+) => (match $self {
        $(Request::$variant(req) => {
            Box::new(req.exec(&$host)
                .and_then(move |res| match res.into_msg($host.handle()) {
                    Ok(m) => future::ok(m),
                    Err(e) => future::err(e),
                }))
        }),+
    });
}

pub trait Executable {
    type Response: FromMessage + IntoMessage;
    type Future: Future<Item = Self::Response, Error = Error>;

    fn exec(self, &Local) -> Self::Future;
}

#[derive(Serialize)]
pub enum Request {
    CommandExec(command::CommandExec),
    PackageInstalled(package::PackageInstalled),
    PackageInstall(package::PackageInstall),
    PackageUninstall(package::PackageUninstall),
    ServiceRunning(service::ServiceRunning),
    ServiceAction(service::ServiceAction),
    ServiceEnabled(service::ServiceEnabled),
    ServiceEnable(service::ServiceEnable),
    ServiceDisable(service::ServiceDisable),
    TelemetryLoad(telemetry::TelemetryLoad),
}

#[derive(Deserialize)]
pub enum RequestValues {
    CommandExec(json::Value),
    PackageInstalled(json::Value),
    PackageInstall(json::Value),
    PackageUninstall(json::Value),
    ServiceRunning(json::Value),
    ServiceAction(json::Value),
    ServiceEnabled(json::Value),
    ServiceEnable(json::Value),
    ServiceDisable(json::Value),
    TelemetryLoad(json::Value),
}

impl Request {
    pub fn exec(self, host: &Local) -> Box<Future<Item = InMessage, Error = Error>> {
        let host = host.clone();

        reqexec!(self, host,
            CommandExec,
            PackageInstalled,
            PackageInstall,
            PackageUninstall,
            ServiceRunning,
            ServiceAction,
            ServiceEnabled,
            ServiceEnable,
            ServiceDisable,
            TelemetryLoad
        )
    }
}

impl FromMessage for Request {
    fn from_msg(mut msg: InMessage) -> Result<Self> {
        let body = msg.take_body();
        let values: RequestValues = json::from_value(msg.into_inner())
            .chain_err(|| "Could not deserialize Request")?;

        let request = match values {
            RequestValues::CommandExec(v) =>
                Request::CommandExec(command::CommandExec::from_msg(partstomsg!(v, body))?),
            RequestValues::PackageInstalled(v) =>
                Request::PackageInstalled(package::PackageInstalled::from_msg(partstomsg!(v, body))?),
            RequestValues::PackageInstall(v) =>
                Request::PackageInstall(package::PackageInstall::from_msg(partstomsg!(v, body))?),
            RequestValues::PackageUninstall(v) =>
                Request::PackageUninstall(package::PackageUninstall::from_msg(partstomsg!(v, body))?),
            RequestValues::ServiceRunning(v) =>
                Request::ServiceRunning(service::ServiceRunning::from_msg(partstomsg!(v, body))?),
            RequestValues::ServiceAction(v) =>
                Request::ServiceAction(service::ServiceAction::from_msg(partstomsg!(v, body))?),
            RequestValues::ServiceEnabled(v) =>
                Request::ServiceEnabled(service::ServiceEnabled::from_msg(partstomsg!(v, body))?),
            RequestValues::ServiceEnable(v) =>
                Request::ServiceEnable(service::ServiceEnable::from_msg(partstomsg!(v, body))?),
            RequestValues::ServiceDisable(v) =>
                Request::ServiceDisable(service::ServiceDisable::from_msg(partstomsg!(v, body))?),
            RequestValues::TelemetryLoad(v) =>
                Request::TelemetryLoad(telemetry::TelemetryLoad::from_msg(partstomsg!(v, body))?),
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
