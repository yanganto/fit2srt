{ self, pkgs, crane, cliRustMsrv, guiRustMsrv }:
let
  cliCraneLib = (crane.mkLib pkgs).overrideToolchain (p: cliRustMsrv);
  guiCraneLib = (crane.mkLib pkgs).overrideToolchain (p: guiRustMsrv);
  src = self;
  doCheck = false;
in
rec {
  default = fit2srt-cli;
  fit2srt-cli = let
    cargoToml = "${self}/cli/Cargo.toml";
    cargoTomlConfig = builtins.fromTOML (builtins.readFile cargoToml);
    version = cargoTomlConfig.package.version;
  in
  cliCraneLib.buildPackage {
    inherit version src cargoToml doCheck;
    name = "fit2srt";
    cargoExtraArgs = "--bin fit2srt-cli";
    cargoArtifacts = cliCraneLib.buildDepsOnly {
      inherit version src cargoToml doCheck;
      name = "fit2srt-cli";
      cargoExtraArgs  = "--bin fit2srt-cli";
    };
  };
  fit2srt-gui = let
    cargoToml = "${self}/gui/Cargo.toml";
    cargoTomlConfig = builtins.fromTOML (builtins.readFile cargoToml);
    version = cargoTomlConfig.package.version;
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
  guiCraneLib.buildPackage {
    inherit version src cargoToml buildInputs nativeBuildInputs doCheck;
    name = "fit2srt";
    cargoExtraArgs = "--bin fit2srt-gui";
    cargoArtifacts = guiCraneLib.buildDepsOnly {
      inherit version src cargoToml buildInputs nativeBuildInputs doCheck;
      name = "fit2srt-gui";
      cargoExtraArgs  = "--bin fit2srt-gui";
    };
  };
  wrapped-fit2srt-gui = pkgs.writeShellScriptBin "wrapped-fit2srt-gui" ''
    export LD_LIBRARY_PATH="$LD_LIBRARY_PATH:${builtins.toString (pkgs.lib.makeLibraryPath [pkgs.wayland])}"
    exec ${fit2srt-gui}/bin/fit2srt-gui "$@"
  '';
}
