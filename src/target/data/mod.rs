// Copyright 2015 Intecture Developers. See the COPYRIGHT file at the
// top-level directory of this distribution and at
// https://intecture.io/COPYRIGHT.
//
// Licensed under the Mozilla Public License 2.0 <LICENSE or
// https://www.tldrlegal.com/l/mpl-2.0>. This file may not be copied,
// modified, or distributed except according to those terms.

mod ffi;

use {Result, OsPlatform};
use std::option::Option as stdOption;
use super::Targets;

pub struct Option<T> {
    target: Targets,
    value: T,
}

impl <T>Option<T> {
    pub fn new(target: Targets, value: T) -> Option<T> {
        Option {
            target: target,
            value: value,
        }
    }
}

pub struct Item<T> {
    centos: stdOption<T>,
    debian: stdOption<T>,
    default: stdOption<T>,
    fedora: stdOption<T>,
    freebsd: stdOption<T>,
    linux: stdOption<T>,
    macos: stdOption<T>,
    redhat: stdOption<T>,
    ubuntu: stdOption<T>,
    unix: stdOption<T>,
}

impl <T>Item<T> {
    pub fn new(options: Vec<Option<T>>) -> Item<T> {
        let mut item = Item {
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
        };

        for opt in options {
            match opt.target {
                Targets::Centos => item.centos = Some(opt.value),
                Targets::Debian => item.debian = Some(opt.value),
                Targets::Default => item.default = Some(opt.value),
                Targets::Fedora => item.fedora = Some(opt.value),
                Targets::Freebsd => item.freebsd = Some(opt.value),
                Targets::Linux => item.linux = Some(opt.value),
                Targets::Macos => item.macos = Some(opt.value),
                Targets::Redhat => item.redhat = Some(opt.value),
                Targets::Ubuntu => item.ubuntu = Some(opt.value),
                Targets::Unix => item.unix = Some(opt.value),
            }
        }

        item
    }

    pub fn resolve(&self, platform: &OsPlatform) -> Result<&T> {
        Ok(match platform {
            &OsPlatform::Centos if self.centos.is_some() => self.centos.as_ref().unwrap(),
            &OsPlatform::Debian if self.debian.is_some() => self.debian.as_ref().unwrap(),
            &OsPlatform::Fedora if self.fedora.is_some() => self.fedora.as_ref().unwrap(),
            &OsPlatform::Freebsd if self.freebsd.is_some() => self.freebsd.as_ref().unwrap(),
            &OsPlatform::Macos if self.macos.is_some() => self.macos.as_ref().unwrap(),
            &OsPlatform::Redhat if self.redhat.is_some() => self.redhat.as_ref().unwrap(),
            &OsPlatform::Ubuntu if self.ubuntu.is_some() => self.ubuntu.as_ref().unwrap(),
            _ => self.default.as_ref().unwrap(),
        })
    }
}

#[macro_export]
macro_rules! dataitem {
    ( $d:ty: $v:expr ) => {{
        use inapi::{DataItem, DataOption, Targets};

        DataItem::new(vec![ DataOption::new(Targets::Default, $v) ]);
    }};

    ( $d:ty: $( $t:path => $v:expr ),* ) => {{
        use inapi::{DataItem, DataOption, Targets};

        let mut options: Vec<DataOption<$d>> = Vec::new();

        $(
            options.push(DataOption::new($t, $v));
        )*

        DataItem::new(options)
    }};
}

// #[cfg(test)]
// mod tests {
//     use super::*;
//
//     #[test]
//     fn test_() {
//     }
// }
