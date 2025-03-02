{
  lib,
  pkgs,
}:
let
  cargoToml = with builtins; (fromTOML (readFile ../Cargo.toml));
in
pkgs.rustPlatform.buildRustPackage rec {
  pname = cargoToml.package.name;
  version = cargoToml.package.version;
  src =
    with lib.fileset;
    toSource {
      root = ../.;
      fileset = unions [
        ../Cargo.toml
        ../Cargo.lock
        ../src
      ];
    };

  cargoLock = {
    lockFile = "${src}/Cargo.lock";
  };

  nativeBuildInputs = with pkgs; [
    rustc
    cargo
    pkg-config
  ];

  buildInputs = with pkgs; [
    openssl
  ];

  meta = with lib; {
    description = "A matrix bot for interacting with self-hosted LLMs.";
    homepage = "https://github.com/tdback/llama-bot";
    license = licenses.mit;
    maintainers = [ ];
  };
}
