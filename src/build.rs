fn main() {
    pkg_config::Config::new()
        .atleast_version("3.2.2")
        .probe("libarchive")
        .expect("Fail to detect the libarchive library");
}
