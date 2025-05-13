{ lib, rustPlatform }:
rustPlatform.buildRustPackage {
  pname = "rdict";
  version = "0.1.0";

  src = lib.cleanSource ./.;

  useFetchCargoVendor = true;
  cargoHash = "sha256-mUjEm9+jPsep473JT3YbSHLHUEA+z5/qbvuqqDeVumY=";

  preCheck = ''
    export HOME="$(mktemp -d)"
  '';

  meta = {
    license = lib.licenses.mit;
  };
}
