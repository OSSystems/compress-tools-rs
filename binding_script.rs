// cargo-deps: bindgen = "0.51.1", pkg-config = "0.3.17"

use std::path::PathBuf;

fn main() {
    let mut lib = pkg_config::Config::new()
        .atleast_version("3.2.2")
        .probe("libarchive")
        .expect("Fail to detect the libarchive library");

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
        .generate()
        .expect("Unable to generate bindings");

    bindings
        .write_to_file(PathBuf::from("src/ffi.rs"))
        .expect("Couldn't write bindings!");

    println!("Sucessfully generated bindings for libarchive.");
}
