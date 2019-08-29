fn main() {
    // This forces the tests to run in a single thread. This is
    // required for use of the mocks as we run mocked binaries.
    println!("cargo:rustc-env=RUST_TEST_THREADS=1");
}
