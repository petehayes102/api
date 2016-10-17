// Copyright 2015-2016 Intecture Developers. See the COPYRIGHT file at the
// top-level directory of this distribution and at
// https://intecture.io/COPYRIGHT.
//
// Licensed under the Mozilla Public License 2.0 <LICENSE or
// https://www.tldrlegal.com/l/mpl-2.0>. This file may not be copied,
// modified, or distributed except according to those terms.

//! Payloads are self-contained projects that encapsulate a specific
//! feature or system function. Think of them as reusable chunks of
//! code that can be run across multiple hosts. Any time you have a
//! task that you want to repeat, it should probably go into a
//! payload.
//!
//! For example, a payload could handle installing a specific
//! package, such as Nginx. Or, you could create a payload that
//! configures iptables.
//!
//! # Examples
//!
//! ```no_run
//! # use inapi::{Host, Payload};
#![cfg_attr(feature = "local-run", doc = "# let mut host = Host::local(Some(\"nodes/mynode.json\")).unwrap();")]
#![cfg_attr(feature = "remote-run", doc = "# let mut host = Host::connect(\"nodes/mynode.json\").unwrap();")]
//! let payload = Payload::new("nginx::install").unwrap(); // format is "payload::executable"
//! payload.run(&mut host, None).unwrap();
//! ```

mod config;
pub mod ffi;

use czmq::{ZMsg, ZPoller, ZSock, ZSys};
use error::{Error, Result};
use host::{Host,HostSendRecv};
use self::config::Config;
use serde_json;
use std::env::{current_dir, set_current_dir};
use std::process::Command;
use std::path::PathBuf;
use std::thread;
use zdaemon::ConfigFile;

#[repr(C)]
#[derive(Clone, Debug, PartialEq, RustcDecodable, RustcEncodable)]
/// The payload's programming language.
pub enum Language {
    C,
    Php,
    Rust,
}

/// Container for running a Payload.
pub struct Payload {
    /// Path to the payload directory.
    path: PathBuf,
    /// Name of the executable/source file to run.
    artifact: String,
    /// Language the payload is written in.
    language: Language,
}

impl Payload {
    /// Create a new Payload using the payload::artifact notation.
    /// This notation is simply "payload" + separator ("::") +
    /// "executable/source file". For example: "nginx::install".
    ///
    /// By default, payloads live in
    /// <project root>/payloads/<payload_name>. Thus the payload name
    /// "nginx" will resolve to <project root>/payloads/nginx/. You
    /// can also specify an absolute path to your payload, which will
    /// override the resolved path.
    ///
    /// ```no_run
    /// # use inapi::Payload;
    /// // Using standard payload/artifact notation...
    /// let payload = Payload::new("iptables::update").unwrap();
    ///
    /// // Using an absolute path...
    /// let payload = Payload::new("/mnt/intecture/payloads/iptables::update").unwrap();
    /// ```
    pub fn new(payload_artifact: &str) -> Result<Payload> {
        let mut parts: Vec<&str> = payload_artifact.split("::").collect();
        let payload = if parts.len() > 0 {
            parts.remove(0)
        } else {
            return Err(Error::Generic("Invalid payload string".into()));
        };
        let artifact = if parts.len() > 1 {
            parts.remove(1)
        } else {
            "default"
        };

        let mut buf = PathBuf::from("payloads");
        buf.push(payload);

        buf.push("payload.json");
        let config = try!(Config::load(&buf));
        buf.pop();

        // Check dependencies
        if let Some(deps) = config.dependencies {
            try!(Self::check_deps(&deps));
        }

        Ok(Payload {
            path: buf,
            artifact: artifact.into(),
            language: config.language,
        })
    }

    /// Compile a payload's source code. This function is also called
    /// by Payload::run(), but is useful for precompiling payloads
    /// ahead of time to catch build errors early.
    ///
    /// Note that this is only useful for compiled languages. If this
    /// function is run on a payload that uses an interpreted
    /// language, it will safely be ignored.
    pub fn build(&self) -> Result<()> {
        let mut make_path = self.path.clone();
        make_path.push("Makefile");

        match self.language {
            Language::C | Language::Rust if make_path.exists() && make_path.is_file() => {
                let current_dir = try!(current_dir());
                try!(set_current_dir(&self.path));

                let output = try!(Command::new("make").output());

                try!(set_current_dir(&current_dir));

                if !output.status.success() {
                    return Err(Error::BuildFailed(try!(String::from_utf8(output.stderr))).into());
                }
            },
            Language::Rust => {
                let manifest_path = format!("{}/Cargo.toml", self.path.to_str().unwrap());
                let output = try!(Command::new("cargo").arg("build").arg("--manifest-path").arg(&manifest_path).output());
                if !output.status.success() {
                    return Err(Error::BuildFailed(try!(String::from_utf8(output.stderr))).into());
                }
            },
            _ => ()
        }

        Ok(())
    }

    /// Execute the payload's artifact.
    ///
    /// For compiled languages, the artifact will be executed
    /// directly. For interpreted languages, the artifact will be
    /// passed as an argument to the interpreter.
    ///
    /// ```no_run
    /// # use inapi::{Host, Payload};
    #[cfg_attr(feature = "local-run", doc = "# let mut host = Host::local(Some(\"nodes/mynode.json\")).unwrap();")]
    #[cfg_attr(feature = "remote-run", doc = "# let mut host = Host::connect(\"nodes/mynode.json\").unwrap();")]
    /// let payload = Payload::new("iptables::configure").unwrap();
    /// payload.run(&mut host, Some(vec![
    ///     "add_rule",
    ///     "..."
    /// ])).unwrap();
    /// ```
    pub fn run(&self, host: &mut Host, user_args: Option<Vec<&str>>) -> Result<()> {
        // Build payload to make sure it's up to date
        try!(self.build());

        let api_endpoint = format!("inproc://payload_{}_{}_api", self.path.to_str().unwrap(), self.artifact);
        let mut api_pipe = try!(ZSock::new_pair(&api_endpoint));
        let file_endpoint = format!("inproc://payload_{}_{}_file", self.path.to_str().unwrap(), self.artifact);
        let mut file_pipe = try!(ZSock::new_pair(&file_endpoint));

        let (mut parent, child) = try!(ZSys::create_pipe());
        let language = self.language.clone();
        let mut payload_path = PathBuf::from(&self.path);
        let artifact = self.artifact.clone();
        let user_args_c: Option<Vec<String>> = match user_args {
            Some(a) => Some(a.into_iter().map(|arg| String::from(arg)).collect()),
            None => None,
        };

        let handle = thread::spawn(move || {
            match language {
                Language::C => {
                    payload_path.push(&artifact);

                    let mut args = vec![api_endpoint, file_endpoint];
                    if let Some(mut a) = user_args_c {
                        args.append(&mut a);
                    }

                    let output = try!(Command::new(payload_path.to_str().unwrap()).args(&args).output());
                    if !output.status.success() {
                        try!(child.signal(0));
                        return Err(Error::RunFailed(try!(String::from_utf8(output.stderr))).into());
                    }
                },
                Language::Php => {
                    payload_path.push(&artifact);
                    if payload_path.extension().is_none() {
                        payload_path.set_extension("php");
                    }

                    let mut args = vec![payload_path.to_str().unwrap().into(), api_endpoint, file_endpoint];
                    if let Some(mut a) = user_args_c {
                        args.append(&mut a);
                    }

                    let output = try!(Command::new("php").args(&args).output());
                    if !output.status.success() {
                        try!(child.signal(0));
                        return Err(Error::RunFailed(try!(String::from_utf8(output.stderr))).into());
                    }
                },
                Language::Rust => {
                    payload_path.push("Cargo.toml");

                    let mut args = vec![
                        "run".into(),
                        "--release".into(),
                        "--bin".into(),
                        artifact,
                        "--manifest-path".into(),
                        payload_path.to_str().unwrap().into(),
                        "--".into(),
                        api_endpoint,
                        file_endpoint
                    ];
                    if let Some(mut a) = user_args_c {
                        args.append(&mut a);
                    }

                    let output = try!(Command::new("cargo").args(&args).output());

                    if !output.status.success() {
                        try!(child.signal(0));
                        return Err(Error::RunFailed(try!(String::from_utf8(output.stderr))).into());
                    }
                }
            }

            try!(child.signal(0));
            Ok(())
        });

        // Send data to payload
        let json = try!(serde_json::to_string(host.data()));
        try!(api_pipe.send_str(&json));

        let mut poller = try!(ZPoller::new());
        try!(poller.add(&mut parent));
        try!(poller.add(&mut api_pipe));
        try!(poller.add(&mut file_pipe));

        loop {
            let sock: Option<ZSock> = poller.wait(None);
            if let Some(mut s) = sock {
                if s == api_pipe || s == file_pipe {
                    let msg = try!(ZMsg::recv(&mut s));
                    try!(host.send(msg));
                }
                else if s == parent {
                    break;
                } else {
                    unreachable!();
                }
            }

            if poller.terminated() {
                break;
            }
        }

        let cmd_result: Result<()> = try!(handle.join());
        try!(cmd_result);

        Ok(())
    }

    fn check_deps(payloads: &[String]) -> Result<()> {
        for payload in payloads {
            try!(Payload::new(payload));
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use host::Host;
    use super::config::Config;
    use std::fs;
    use std::io::Write;
    use std::path::PathBuf;
    use std::process::Command;
    use super::*;
    use tempdir::TempDir;
    use zdaemon::ConfigFile;

    #[test]
    fn test_new_nodeps() {
        let _ = ::_MOCK_ENV.init();

        let tempdir = TempDir::new("test_payload_new_nodeps").unwrap();
        let mut buf = tempdir.path().to_owned();

        create_cargo_proj(&mut buf);

        let conf = Config {
            author: "Dr. Hibbert".into(),
            repository: "https://github.com/dhibbz/hehehe.git".into(),
            language: Language::Rust,
            dependencies: Some(vec!["missing_payload".into()]),
        };

        buf.push("payload.json");
        conf.save(&buf).unwrap();
        buf.pop();

        assert!(Payload::new(buf.to_str().unwrap()).is_err());
    }

    #[test]
    fn test_build_rust() {
        let _ = ::_MOCK_ENV.init();

        let tempdir = TempDir::new("test_payload_build_rust").unwrap();
        let mut buf = tempdir.path().to_owned();

        create_cargo_proj(&mut buf);

        let conf = Config {
            author: "Dr. Hibbert".into(),
            repository: "https://github.com/dhibbz/hehehe.git".into(),
            language: Language::Rust,
            dependencies: None,
        };

        buf.push("payload.json");
        conf.save(&buf).unwrap();
        buf.pop();

        let payload = Payload::new(buf.to_str().unwrap()).unwrap();
        payload.build().unwrap();
    }

    #[test]
    fn test_build_c() {
        let _ = ::_MOCK_ENV.init();

        let tempdir = TempDir::new("test_payload_build_c").unwrap();
        let mut buf = tempdir.path().to_owned();

        buf.push("Makefile");
        let mut fh = fs::File::create(&buf).unwrap();
        fh.write_all(b"all:
\ttouch test").unwrap();
        buf.pop();

        let conf = Config {
            author: "Dr. Hibbert".into(),
            repository: "https://github.com/dhibbz/hehehe.git".into(),
            language: Language::C,
            dependencies: None,
        };

        buf.push("payload.json");
        conf.save(&buf).unwrap();
        buf.pop();

        let payload = Payload::new(buf.to_str().unwrap()).unwrap();
        payload.build().unwrap();

        buf.push("test");
        assert!(buf.exists());
    }

    #[test]
    fn test_run() {
        let _ = ::_MOCK_ENV.init();

        let tempdir = TempDir::new("test_payload_run").unwrap();
        let mut buf = tempdir.path().to_owned();

        create_cargo_proj(&mut buf);

        let conf = Config {
            author: "Dr. Hibbert".into(),
            repository: "https://github.com/dhibbz/hehehe.git".into(),
            language: Language::Rust,
            dependencies: None,
        };

        buf.push("payload.json");
        conf.save(&buf).unwrap();
        buf.pop();

        let mut host = Host::test_new(None, None, None, None);
        let payload = Payload::new(buf.to_str().unwrap()).unwrap();
        payload.run(&mut host, Some(vec!["abc"])).unwrap();
    }

    fn create_cargo_proj(buf: &mut PathBuf) {
        let output = Command::new("cargo")
                             .args(&["init", buf.to_str().unwrap(), "--bin", "--name", "default"])
                             .output()
                             .expect("Failed to execute process");
        assert!(output.status.success());
    }
}
