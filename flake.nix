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
        cliCargoToml = (builtins.fromTOML (builtins.readFile ./cli/Cargo.toml));
        guiCargoToml = (builtins.fromTOML (builtins.readFile ./gui/Cargo.toml));
      in
      {
        packages = (import ./nix/packages.nix { 
          inherit self pkgs crane;
          cliRustMsrv = pkgs.rust-bin.stable.${cliCargoToml.package.rust-version}.minimal;
          guiRustMsrv = pkgs.rust-bin.stable.${guiCargoToml.package.rust-version}.minimal;
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
              pkgs.rust-bin.stable.${guiCargoToml.package.rust-version}.minimal 
            ];
            inherit nativeBuildInputs;
            # ICED needed additional LD_LIBRARY_PATH
            shellHook = ''
              export LD_LIBRARY_PATH="$LD_LIBRARY_PATH:${builtins.toString (pkgs.lib.makeLibraryPath [pkgs.wayland])}"
            '';
          });
          ci = pkgs.mkShell ({
            buildInputs = buildInputs ++ [
              pkgs.rust-bin.stable.${guiCargoToml.package.rust-version}.default 
            ];
            inherit nativeBuildInputs;
          });
        };
      }
    );
}
