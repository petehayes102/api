// Copyright 2015 Intecture Developers. See the COPYRIGHT file at the
// top-level directory of this distribution and at
// https://intecture.io/COPYRIGHT.
//
// Licensed under the Mozilla Public License 2.0 <LICENSE or
// https://www.tldrlegal.com/l/mpl-2.0>. This file may not be copied,
// modified, or distributed except according to those terms.

use {Host, Result};
use super::Targets;

pub struct Item<T> {
    centos: Option<T>,
    debian: Option<T>,
    default: Option<T>,
    fedora: Option<T>,
    freebsd: Option<T>,
    linux: Option<T>,
    macos: Option<T>,
    redhat: Option<T>,
    ubuntu: Option<T>,
    unix: Option<T>,
}

impl <T>Item<T> {
    pub fn new() -> Item<T> {
        Item {
            centos: None,
            debian: None,
            default: None,
            fedora: None,
            freebsd: None,
            linux: None,
            macos: None,
            redhat: None,
            ubuntu: None,
            unix: None,
        }
    }

    pub fn add(&mut self, item: T, target: Option<Targets>) {
        if let Some(t) = target {
            match t {
                Targets::Centos => self.centos = Some(item),
                Targets::Debian => self.debian = Some(item),
                Targets::Fedora => self.fedora = Some(item),
                Targets::Freebsd => self.freebsd = Some(item),
                Targets::Linux => self.linux = Some(item),
                Targets::Macos => self.macos = Some(item),
                Targets::Redhat => self.redhat = Some(item),
                Targets::Ubuntu => self.ubuntu = Some(item),
                Targets::Unix => self.unix = Some(item),
            }
        } else {
            self.default = Some(item);
        }
    }

    pub fn resolve(&self, host: &mut Host) -> Result<Option<&T>> {
        let os = try!(host.get_telemetry_os());

        let data = match os {
            Targets::Centos if self.centos.is_some() => self.centos.as_ref(),
            Targets::Debian if self.debian.is_some() => self.debian.as_ref(),
            Targets::Fedora if self.fedora.is_some() => self.fedora.as_ref(),
            Targets::Freebsd if self.freebsd.is_some() => self.freebsd.as_ref(),
            Targets::Linux if self.linux.is_some() => self.linux.as_ref(),
            Targets::Macos if self.macos.is_some() => self.macos.as_ref(),
            Targets::Redhat if self.redhat.is_some() => self.redhat.as_ref(),
            Targets::Ubuntu if self.ubuntu.is_some() => self.ubuntu.as_ref(),
            Targets::Unix if self.unix.is_some() => self.unix.as_ref(),
            _ => self.default.as_ref(),
        };

        Ok(data)
    }
}

// #[cfg(test)]
// mod tests {
//     use super::*;
//
//     #[test]
//     fn test_() {
//     }
// }
