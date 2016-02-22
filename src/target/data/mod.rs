// Copyright 2015 Intecture Developers. See the COPYRIGHT file at the
// top-level directory of this distribution and at
// https://intecture.io/COPYRIGHT.
//
// Licensed under the Mozilla Public License 2.0 <LICENSE or
// https://www.tldrlegal.com/l/mpl-2.0>. This file may not be copied,
// modified, or distributed except according to those terms.

use Host;

enum Option<T> {
    Centos(T),
    Debian(T),
    Default(T),
    Fedora(T),
    Freebsd(T),
    Linux(T),
    Macos(T),
    Redhat(T),
    Ubuntu(T),
    Unix(T),
}

struct Item<T> {
    options: [Option<T>]
}

impl <T>Item<T> {
    fn new(options: [Option<T>]) -> Item<T> {
        Item {
            options: options,
        }
    }

    fn resolve(&self, host: &mut Host) -> T {
        let os = "xxx";

        for opt in self.options {
            match opt {
                Option::Centos(data) if os == "centos" => return data,
                Option::Debian(data) if os == "debian" => return data,
                Option::Fedora(data) if os == "fedora" => return data,
                Option::Freebsd(data) if os == "freebsd" => return data,
                Option::Linux(data) if os == "linux" => return data,
                Option::Macos(data) if os == "macos" => return data,
                Option::Redhat(data) if os == "redhat" => return data,
                Option::Ubuntu(data) if os == "ubuntu" => return data,
                Option::Unix(data) if os == "unix" => return data,
                Option::Default(data) => return data,
            }
        }
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
