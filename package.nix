{ lib, rustPlatform }:
rustPlatform.buildRustPackage {
  pname = "rdict";
  version = "0.1.0";

  src = lib.cleanSource ./.;

  useFetchCargoVendor = true;
  cargoHash = "sha256-BT0+N3PK+d20RLEhPRhcrZ6F9sZ63OmyOWdjdVq48Mk=";

  preCheck = ''
    export HOME="$(mktemp -d)"
  '';

  meta = {
    license = lib.licenses.mit;
  };
}
