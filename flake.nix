{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-25.05";

    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
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
        devShells = 
        let 
            buildInputs = with pkgs; [ 
              wayland
              libxkbcommon
            ];
            nativeBuildInputs = with pkgs; [ 
              pkg-config
              wayland
              libxkbcommon
              zenity
            ];
        in
        rec {
          default = dev;
          dev = pkgs.mkShell ({
            buildInputs = buildInputs ++ [
              pkgs.rust-bin.stable.${cargoToml.package.rust-version}.minimal 
            ];
            inherit nativeBuildInputs;
            # ICED needed additional LD_LIBRARY_PATH
            shellHook = ''
              export LD_LIBRARY_PATH="$LD_LIBRARY_PATH:${builtins.toString (pkgs.lib.makeLibraryPath [pkgs.wayland])}"
            '';
          });
          ci = pkgs.mkShell ({
            buildInputs = buildInputs ++ [
              pkgs.rust-bin.stable.${cargoToml.package.rust-version}.default 
            ];
            inherit nativeBuildInputs;
          });
        };
      }
    );
}
