{ self, pkgs, crane, specificRust }:
let
  craneLib = (crane.mkLib pkgs).overrideToolchain (p: specificRust);
  cargoToml = "${self}/Cargo.toml";
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
  default = fit2srt;
  fit2srt = craneLib.buildPackage {
    inherit version src env cargoToml buildInputs nativeBuildInputs outputHashes doCheck;
    name = "fit2srt";
    cargoExtraArgs = "";
    cargoArtifacts = craneLib.buildDepsOnly {
      inherit version src env cargoToml buildInputs nativeBuildInputs outputHashes doCheck;
      name = "fit2srt";
      cargoExtraArgs  = "";
    };
  };
}
