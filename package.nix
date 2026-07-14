{
  lib,
  stdenv,
  rustPlatform,
  installShellFiles,
  writableTmpDirAsHomeHook,
}:
let
  version = "0.3.0";
  src = lib.cleanSource ./.;
in
rec {
  default = rdict;

  rdict = rustPlatform.buildRustPackage {
    inherit version src;

    pname = "rdict";

    cargoLock = {
      lockFile = ./Cargo.lock;
    };

    buildAndTestSubdir = "./rdict-cli";

    nativeBuildInputs = lib.optionals (stdenv.buildPlatform.canExecute stdenv.hostPlatform) [
      installShellFiles
      writableTmpDirAsHomeHook
    ];

    postInstall = lib.optionalString (stdenv.buildPlatform.canExecute stdenv.hostPlatform) ''
      installShellCompletion --cmd rdict \
        --bash <("$out/bin/rdict" --completion bash) \
        --zsh <("$out/bin/rdict" --completion zsh) \
        --fish <("$out/bin/rdict" --completion fish)
    '';

    meta = {
      license = lib.licenses.mit;
      mainProgram = "rdict";
    };
  };

  rdict-telegram = rustPlatform.buildRustPackage {
    inherit version src;

    pname = "rdict-telegram";

    cargoLock = {
      lockFile = ./Cargo.lock;
    };

    nativeBuildInputs = lib.optionals (stdenv.buildPlatform.canExecute stdenv.hostPlatform) [
      writableTmpDirAsHomeHook
    ];

    buildAndTestSubdir = "./rdict-telegram";

    meta = {
      license = lib.licenses.mit;
      mainProgram = "rdict-telegram";
    };
  };

  # TODO: rdict-iced
}
