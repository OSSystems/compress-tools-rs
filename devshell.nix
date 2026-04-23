{ pkgs, lib, inputs, system, ... }:

let
  rust-toolchain = with inputs.rust.packages.${system};
    let
      msrvToolchain = toolchainOf {
        channel = (lib.importTOML ./Cargo.toml).package.rust-version;
        sha256 = "sha256-yMuSb5eQPO/bHv+Bcf/US8LVMbf/G/0MSfiPwBhiPpk=";
      };
    in
    combine [
      (msrvToolchain.withComponents [ "rustc" "cargo" "rust-src" "clippy" ])

      latest.rustfmt
      latest.rust-analyzer
    ];
in
pkgs.mkShell {
  buildInputs = with pkgs; [
    rust-toolchain
    rust-bindgen
    pkg-config
    libarchive
    # Required by the `static` feature (build.rs probes these via
    # pkg-config when libarchive is linked statically).
    libb2
    lz4
    zstd
    xz
    bzip2
    zlib
    libxml2
    openssl
    clang
    llvmPackages.libclang

    cargo-release
  ];

  shellHook = ''
    export LIBCLANG_PATH="${pkgs.llvmPackages.libclang}/lib";
  '';
}
