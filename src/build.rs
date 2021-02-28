// Copyright (C) 2019-2021 O.S. Systems Software LTDA
//
// SPDX-License-Identifier: MIT OR Apache-2.0

fn main() {
    find_libarchive()
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
}

#[cfg(feature = "generate_ffi")]
fn update_bindings() -> Result<(), Box<dyn std::error::Error>> {
    use std::path::PathBuf;

    let mut lib = find_libarchive()?;

    let include_path = lib
        .include_paths
        .pop()
        .unwrap_or(PathBuf::from("usr/include"));

    let bindings = bindgen::Builder::default()
        // Set rustfmt setting
        .rustfmt_configuration_file(Some(".rustfmt.toml".into()))

        // Set include path
        .header(format!("{}/archive.h", include_path.display()))
        .header(format!("{}/archive_entry.h", include_path.display()))

        // We need to add this as raw_line to pass cargo clippy warning about
        // convert to upper camel case
        .raw_line("#![allow(non_camel_case_types)]\n")

        // We need to add this as raw_line otherwise bindgen generates this as
        // u32, causing type mismatch
        .raw_line("pub const ARCHIVE_EOF: i32 = 1;")
        .raw_line("pub const ARCHIVE_OK: i32 = 0;")

        // Binding whitelist
        .whitelist_var("ARCHIVE_EXTRACT_TIME")
        .whitelist_var("ARCHIVE_EXTRACT_PERM")
        .whitelist_var("ARCHIVE_EXTRACT_ACL")
        .whitelist_var("ARCHIVE_EXTRACT_FFLAGS")
        .whitelist_var("ARCHIVE_EXTRACT_OWNER")
        .whitelist_var("ARCHIVE_EXTRACT_FFLAGS")
        .whitelist_var("ARCHIVE_EXTRACT_XATTR")
        .whitelist_function("archive_read_new")
        .whitelist_function("archive_read_set_seek_callback")
        .whitelist_function("archive_read_support_filter_all")
        .whitelist_function("archive_read_support_format_all")
        .whitelist_function("archive_read_support_format_raw")
        .whitelist_function("archive_read_close")
        .whitelist_function("archive_read_free")
        .whitelist_function("archive_read_data_block")
        .whitelist_function("archive_read_next_header")
        .whitelist_function("archive_read_open")
        .whitelist_function("archive_write_disk_new")
        .whitelist_function("archive_write_disk_set_options")
        .whitelist_function("archive_write_disk_set_standard_lookup")
        .whitelist_function("archive_write_header")
        .whitelist_function("archive_write_finish_entry")
        .whitelist_function("archive_write_data_block")
        .whitelist_function("archive_write_close")
        .whitelist_function("archive_write_free")
        .whitelist_function("archive_entry_pathname")
        .whitelist_function("archive_entry_free")
        .whitelist_function("archive_entry_set_pathname")
        .whitelist_function("archive_entry_set_hardlink")
        .whitelist_function("archive_entry_hardlink")
        .whitelist_function("archive_set_error")
        .whitelist_function("archive_error_string")
        .whitelist_function("archive_errno")
        .generate()
        .expect("Unable to generate bindings");

    bindings.write_to_file(PathBuf::from("src/ffi.rs"))?;

    Ok(())
}
