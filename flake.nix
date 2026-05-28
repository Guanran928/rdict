{
  description = "CLI youdao dictionary tool";

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs =
    {
      self,
      nixpkgs,
      flake-utils,
    }:
    flake-utils.lib.eachDefaultSystem (
      system:
      let
        pkgs = import nixpkgs {
          inherit system;
        };
        inherit (pkgs) lib;

        rustPlatform = pkgs.makeRustPlatform {
          cargo = pkgs.cargo;
          rustc = pkgs.rustc;
        };

        rustToolchain =
          with pkgs;
          [
            rustc
            rustfmt
            rust-analyzer
            cargo
          ]
          ++ (lib.optionals pkgs.stdenv.isDarwin [ pkgs.libiconv ]);

      in
      {
        devShells.default = pkgs.mkShell {
          packages = rustToolchain;
        };

        packages = {
          default = rustPlatform.buildRustPackage {
            pname = "rdict";
            version = "0.1.0";
            src = ./.;
            cargoLock = {
              lockFile = ./Cargo.lock;
            };
            buildInputs = lib.optionals pkgs.stdenv.isDarwin [ pkgs.libiconv ];
          };

        };
      }
    );
}
