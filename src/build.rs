fn main() {
    find_libarchive()
}

#[cfg(not(windows))]
fn find_libarchive() {
    pkg_config::Config::new()
        .atleast_version("3")
        .probe("libarchive")
        .expect("Unable to find libarchive");
}

#[cfg(windows)]
fn find_libarchive() {
    vcpkg::Config::new()
        .find_package("libarchive")
        .expect("Unable to find libarchive");
}
