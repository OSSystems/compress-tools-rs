// Copyright (C) 2019-2021 O.S. Systems Software LTDA
//
// SPDX-License-Identifier: MIT OR Apache-2.0

fn main() {
    find_libarchive();
}

#[cfg(not(target_env = "msvc"))]
fn find_libarchive() {
    let mode = if cfg!(feature = "static") {
        "static"
    } else {
        "dylib"
    };

    if mode == "static" {
        link_deps(mode);
    }

    link_libarchive(mode);
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

#[cfg(not(target_env = "msvc"))]
fn link_libarchive(mode: &str) {
    let libarchive = pkg_config::Config::new()
        .atleast_version("3.2.0")
        .statik(mode == "static")
        .probe("libarchive")
        .expect("Unable to find libarchive");

    for link_path in libarchive.link_paths {
        println!("cargo:rustc-link-search=native={}", link_path.display());
    }

    for lib in libarchive.libs {
        println!("cargo:rustc-link-lib={}={}", mode, lib);
    }
}

#[cfg(not(target_env = "msvc"))]
fn link_deps(mode: &str) {
    find_link_paths();

    #[cfg(target_os = "linux")]
    println!("cargo:rustc-link-lib=dylib=stdc++");

    #[cfg(target_os = "macos")]
    {
        println!("cargo:rustc-link-search=native=/usr/local/opt/bzip2/lib");
        link_expat(mode);
        link_iconv(mode);
    }

    link_icuuc(mode);
}

#[cfg(not(target_env = "msvc"))]
fn find_link_paths() {
    let pc_path = pkg_config::get_variable("pkg-config", "pc_path").expect("failed to get pc_path");

    for path in pc_path.split(":") {
        println!(
            "cargo:rustc-link-search=native={}",
            path.replace("/pkgconfig", "")
        );
    }

    if let Ok(pkg_config_path) = std::env::var("PKG_CONFIG_PATH") {
        for path in pkg_config_path.split(":") {
            println!(
                "cargo:rustc-link-search=native={}",
                path.replace("/pkgconfig", "")
            );
        }
    }
}

#[cfg(not(target_env = "msvc"))]
fn link(name: &str, mode: &str) -> pkg_config::Library {
    let lib = pkg_config::Config::new()
        .statik(mode == "static")
        .probe(name)
        .expect(format!("unable to find {}", name).as_str());

    for link_path in lib.link_paths.iter() {
        println!("cargo:rustc-link-search=native={}", link_path.display());
    }

    lib
}

#[cfg(target_os = "macos")]
fn link_expat(mode: &str) {
    let expat = link("expat", mode);

    for lib in expat.libs {
        println!("cargo:rustc-link-lib={}={}", mode, lib);
    }
}

#[cfg(not(target_env = "msvc"))]
fn link_icuuc(mode: &str) {
    let _ = link("icu-uc", mode);

    println!("cargo:rustc-link-lib={}=icuuc", mode);
    println!("cargo:rustc-link-lib={}=icudata", mode);
}

#[cfg(target_os = "macos")]
fn link_iconv(mode: &str) {
    if mode == "static" {
        println!("cargo:rustc-link-search=/usr/local/opt/libiconv/lib/")
    }
}
