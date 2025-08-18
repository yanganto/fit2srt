{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-25.05";

    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs = {
        nixpkgs.follows = "nixpkgs";
        flake-utils.follows = "flake-utils";
      };
    };

    flake-utils.url = "github:numtide/flake-utils";
    crane.url = "github:ipetkov/crane";
  };

  outputs = { self, nixpkgs, rust-overlay, flake-utils, crane, ... }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        overlays = [ (import rust-overlay) ];
        pkgs = import nixpkgs {
          inherit system overlays;
        };
        cargoToml = (builtins.fromTOML (builtins.readFile ./cli/Cargo.toml));
      in
      {
        packages = (import ./nix/packages.nix { 
          inherit self pkgs crane;
          specificRust = pkgs.rust-bin.stable.${cargoToml.package.rust-version}.minimal;
        });
        devShells = rec {
          default = dev;
          dev = pkgs.mkShell ({
            buildInputs = [ pkgs.rust-bin.stable.${cargoToml.package.rust-version}.minimal ];
          });
          ci = pkgs.mkShell ({
            buildInputs = [ pkgs.rust-bin.stable.latest.default ];
          });
        };
      }
    );
}
