// Copyright (C) 2019-2021 O.S. Systems Software LTDA
//
// SPDX-License-Identifier: MIT OR Apache-2.0

fn main() {
    find_libarchive();
}

#[cfg(not(target_env = "msvc"))]
fn find_libarchive() {
    const MACOS_HOMEBREW_LIBARCHIVE_PATH: &str = "/opt/homebrew/opt/libarchive/lib/pkgconfig/";

    if cfg!(target_os = "macos")
        && pkg_config::Config::new()
            .atleast_version("3.2.0")
            .probe("libarchive")
            .is_err()
        && std::path::Path::new(MACOS_HOMEBREW_LIBARCHIVE_PATH).exists()
    {
        // on OSX brew doesn't install libarchive in the default path...
        // try that workaround as it's a pain providing this in the env e.g.
        // for vs code usage.
        // todo should add to current one and set afterwards to current value!
        std::env::set_var("PKG_CONFIG_PATH", MACOS_HOMEBREW_LIBARCHIVE_PATH);
    }

    if cfg!(feature = "static") {
        pkg_config::Config::new()
            .statik(cfg!(feature = "static"))
            .probe("libb2")
            .expect("Unable to find libb2");

        pkg_config::Config::new()
            .statik(cfg!(feature = "static"))
            .probe("liblz4")
            .expect("Unable to find liblz4");

        pkg_config::Config::new()
            .statik(cfg!(feature = "static"))
            .probe("libzstd")
            .expect("Unable to find libzstd");
    }

    pkg_config::Config::new()
        .atleast_version("3.2.0")
        .statik(cfg!(feature = "static"))
        .probe("libarchive")
        .expect("Unable to find libarchive");

    if cfg!(feature = "static") {
        println!("cargo:rustc-link-lib=static=archive");
    }
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
