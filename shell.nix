{ pkgs ? import <nixpkgs> {} }:

with pkgs;

stdenv.mkDerivation {
  name = "compress-tools";
  buildInputs = [
    rust-bindgen
    pkg-config
    libarchive
    clang
    llvmPackages.libclang
  ];
  # why do we need to set the library path manually?
  shellHook = ''
      export LIBCLANG_PATH="${llvmPackages.libclang}/lib";
  '';
}
