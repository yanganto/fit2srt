{ self, pkgs, crane, specificRust }:
let
  craneLib = (crane.mkLib pkgs).overrideToolchain (p: specificRust);
  cargoToml = "${self}/cli/Cargo.toml";
  cargoTomlConfig = builtins.fromTOML (builtins.readFile cargoToml);
  version = cargoTomlConfig.package.version;
  src = self;
  buildInputs = [ ];
  nativeBuildInputs = [ ];
  outputHashes = { };
  doCheck = false;
  env = { };
in
rec {
  default = fit2srt-cli;
  fit2srt-cli = craneLib.buildPackage {
    inherit version src env cargoToml buildInputs nativeBuildInputs outputHashes doCheck;
    name = "fit2srt";
    cargoExtraArgs = "--bin fit2srt-cli";
    cargoArtifacts = craneLib.buildDepsOnly {
      inherit version src env cargoToml buildInputs nativeBuildInputs outputHashes doCheck;
      name = "fit2srt-cli";
      cargoExtraArgs  = "--bin fit2srt-cli";
    };
  };
}
