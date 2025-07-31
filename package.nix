{ lib, rustPlatform }:
let
  cargoHash = "sha256-9AsvTTPM4ru88DGIHkxSWfhBEsRhovTQoWNHUUKpllE=";
in
rec {
  default = rdict;
  rdict = rustPlatform.buildRustPackage {
    inherit cargoHash;

    pname = "rdict";
    version = "0.2.0";

    src = lib.cleanSource ./.;

    buildAndTestSubdir = "./rdict-cli";

    preCheck = ''
      export HOME="$(mktemp -d)"
    '';

    meta = {
      license = lib.licenses.mit;
      mainProgram = "rdict";
    };
  };

  rdict-telegram = rustPlatform.buildRustPackage {
    inherit cargoHash;

    pname = "rdict-telegram";
    version = "0.2.0";

    src = lib.cleanSource ./.;

    buildAndTestSubdir = "./rdict-telegram";

    preCheck = ''
      export HOME="$(mktemp -d)"
    '';

    meta = {
      license = lib.licenses.mit;
      mainProgram = "rdict-telegram";
    };
  };
}
