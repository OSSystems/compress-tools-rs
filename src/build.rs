fn main() {
    if !find_libarchive() {
        panic!("Unable to find libarchive");
    }
}

#[cfg(not(windows))]
fn find_libarchive() -> bool {
    pkg_config::Config::new()
        .atleast_version("3")
        .probe("libarchive")
        .is_ok()
}

#[cfg(windows)]
fn find_libarchive() -> bool {
    vcpkg::Config::new().find_package("libarchive").is_ok()
}
