{ lib, rustPlatform }:
rec {
  default = rdict;
  rdict = rustPlatform.buildRustPackage {
    pname = "rdict";
    version = "0.1.0";

    src = lib.cleanSource ./.;

    useFetchCargoVendor = true;
    cargoHash = "sha256-odZ6hTB6OyC7LpJZL1kWSLaa4kA9R/TvD1MCM581ik8=";

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
    pname = "rdict-telegram";
    version = "0.1.0";

    src = lib.cleanSource ./.;

    useFetchCargoVendor = true;
    cargoHash = "sha256-odZ6hTB6OyC7LpJZL1kWSLaa4kA9R/TvD1MCM581ik8=";

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
