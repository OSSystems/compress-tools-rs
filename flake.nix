{
  description = "compress-tools-rs";

  inputs = {
    nixpkgs.url = "nixpkgs/nixos-24.05";
    flake-utils.url = "github:numtide/flake-utils";

    rust = {
      url = "github:nix-community/fenix";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs = { self, nixpkgs, flake-utils, rust }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = nixpkgs.legacyPackages.${system};

        rust-toolchain = with rust.packages.${system};
          let
            msrvToolchain = toolchainOf {
              channel = "1.82.0";
              sha256 = "sha256-yMuSb5eQPO/bHv+Bcf/US8LVMbf/G/0MSfiPwBhiPpk=";
            };
          in
          combine [
            (msrvToolchain.withComponents [ "rustc" "cargo" "rust-src" "clippy" ])

            latest.rustfmt
            latest.rust-analyzer
          ];
      in
      {
        devShells.default = pkgs.mkShell {
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
            clang
            llvmPackages.libclang

            cargo-release
          ];

          # why do we need to set the library path manually?
          shellHook = ''
            export LIBCLANG_PATH="${pkgs.llvmPackages.libclang}/lib";
          '';
        };
      });
}
