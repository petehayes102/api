// Copyright 2015-2017 Intecture Developers.
//
// Licensed under the Mozilla Public License 2.0 <LICENSE or
// https://www.tldrlegal.com/l/mpl-2.0>. This file may not be copied,
// modified, or distributed except according to those terms.

use bytes::Bytes;
use errors::*;
use serde_json::Value;
use std::io;
use tokio_core::reactor::Handle;
use tokio_proto::streaming::{Body, Message};

pub type InMessage = Message<Value, Body<Bytes, io::Error>>;

// @todo This trait might disappear when TryFrom is stabilised.
// https://github.com/rust-lang/rust/issues/33417
pub trait FromMessage {
    fn from_msg(InMessage) -> Result<Self> where Self: Sized;
}

// @todo This trait might disappear when TryFrom is stabilised.
// https://github.com/rust-lang/rust/issues/33417
pub trait IntoMessage {
    fn into_msg(self, &Handle) -> Result<InMessage>;
}
