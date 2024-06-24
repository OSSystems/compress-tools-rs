{
  description = "compress-tools-rs";

  inputs = {
    nixpkgs.url = "nixpkgs/nixos-23.05";
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
              channel = "1.65.0";
              sha256 = "sha256-DzNEaW724O8/B8844tt5AVHmSjSQ3cmzlU4BP90oRlY=";
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
