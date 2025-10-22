{
  inputs = {
    naersk.url = "github:nix-community/naersk/master";
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    utils.url = "github:numtide/flake-utils";
  };

  outputs = { nixpkgs, utils, naersk, ... }:
    utils.lib.eachDefaultSystem (system:
      let
        pkgs = import nixpkgs { inherit system; };
        naersk-lib = pkgs.callPackage naersk { };
      in {
        defaultPackage = naersk-lib.buildPackage {
          src = ./.;
          # The binary is called `maconomy`, not `maconomy-cli`
          pname = "maconomy";
        };
        devShell = with pkgs;
          mkShell {
            buildInputs = [
              # Rust stuff
              cargo
              rustc
              rustfmt
              pre-commit
              rustPackages.clippy

              # Other
              openssl
              pkg-config
              chromium
            ];
            RUST_SRC_PATH = rustPlatform.rustLibSrc;
          };
      });
}
