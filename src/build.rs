// Copyright (C) 2019-2021 O.S. Systems Software LTDA
//
// SPDX-License-Identifier: MIT OR Apache-2.0

fn main() {
    find_libarchive();
}

#[cfg(not(target_env = "msvc"))]
fn find_libarchive() {
    pkg_config::Config::new()
        .atleast_version("3.2.0")
        .probe("libarchive")
        .expect("Unable to find libarchive");
}

#[cfg(target_env = "msvc")]
fn find_libarchive() {
    vcpkg::Config::new()
        .find_package("libarchive")
        .expect("Unable to find libarchive");

    println!("cargo:rustc-link-lib=static=archive");
    println!("cargo:rustc-link-lib=User32");
    println!("cargo:rustc-link-lib=Crypt32");
}
