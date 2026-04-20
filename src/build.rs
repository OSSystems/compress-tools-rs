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

    let probe_static = |pc_name: &str| {
        pkg_config::Config::new()
            .statik(true)
            .probe(pc_name)
            .unwrap_or_else(|e| panic!("Unable to find {pc_name}: {e}"));
    };

    if cfg!(feature = "static_b2") {
        probe_static("libb2");
    }
    if cfg!(feature = "static_lz4") {
        probe_static("liblz4");
    }
    if cfg!(feature = "static_zstd") {
        probe_static("libzstd");
    }
    if cfg!(feature = "static_lzma") {
        probe_static("liblzma");
    }
    if cfg!(feature = "static_bz2") {
        probe_static("bzip2");
    }
    if cfg!(feature = "static_z") {
        probe_static("zlib");
    }
    if cfg!(feature = "static_xml2") {
        probe_static("libxml-2.0");
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
    if cfg!(feature = "win_user32") {
        println!("cargo:rustc-link-lib=User32");
    }
    if cfg!(feature = "win_crypt32") {
        println!("cargo:rustc-link-lib=Crypt32");
    }
    if cfg!(feature = "win_advapi32") {
        println!("cargo:rustc-link-lib=advapi32");
    }
    if cfg!(feature = "win_xmllite") {
        println!("cargo:rustc-link-lib=xmllite");
    }
}
