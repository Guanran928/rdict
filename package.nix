{ lib, rustPlatform }:
rec {
  default = rdict;
  rdict = rustPlatform.buildRustPackage {
    pname = "rdict";
    version = "0.1.0";

    src = lib.cleanSource ./.;

    useFetchCargoVendor = true;
    cargoHash = "sha256-yoEdAB82lVqzn+GSTPi4FMa+xaNUdejPOKn7ccrpk2U=";

    buildAndTestSubdir = "./rdict-cli";

    preCheck = ''
      export HOME="$(mktemp -d)"
    '';

    meta = {
      license = lib.licenses.mit;
    };
  };

  rdict-telegram = rustPlatform.buildRustPackage {
    pname = "rdict";
    version = "0.1.0";

    src = lib.cleanSource ./.;

    useFetchCargoVendor = true;
    cargoHash = "sha256-yoEdAB82lVqzn+GSTPi4FMa+xaNUdejPOKn7ccrpk2U=";

    buildAndTestSubdir = "./rdict-telegram";

    preCheck = ''
      export HOME="$(mktemp -d)"
    '';

    meta = {
      license = lib.licenses.mit;
    };
  };
}
