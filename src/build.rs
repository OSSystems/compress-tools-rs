fn main() {
    // This forces the tests to run in a single thread. This is
    // required for use of the mocks as we run mocked binaries.
    pkg_config::Config::new()
        .atleast_version("3.2.2")
        .probe("libarchive")
        .expect("Fail to detect the libarchive library");
}
