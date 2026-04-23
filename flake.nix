{
  description = "compress-tools-rs";

  inputs = {
    nixpkgs.url = "nixpkgs/nixos-25.11";
    red-tape = {
      url = "github:phaer/red-tape";
      inputs.nixpkgs.follows = "nixpkgs";
    };

    rust = {
      url = "github:nix-community/fenix";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs = inputs: inputs.red-tape.mkFlake {
    inherit inputs;
    src = ./.;
  };
}
