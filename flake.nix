{
  description = "compress-tools-rs";

  inputs = {
    nixpkgs.url = "nixpkgs/nixos-22.05";
    flake-utils = {
      url = "github:numtide/flake-utils";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs = { self, nixpkgs, flake-utils }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = nixpkgs.legacyPackages.${system};
      in
      {
        devShell = pkgs.mkShell {
          buildInputs = with pkgs; [
            rust-bindgen
            pkg-config
            libarchive
            clang
            llvmPackages.libclang
          ];

          # why do we need to set the library path manually?
          shellHook = ''
            export LIBCLANG_PATH="${pkgs.llvmPackages.libclang}/lib";
          '';
        };
      });
}
