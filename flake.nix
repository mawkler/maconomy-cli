{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    naersk = {
      url = "github:nix-community/naersk";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs =
    {
      flake-utils,
      naersk,
      nixpkgs,
      ...
    }:
    flake-utils.lib.eachDefaultSystem (
      system:
      let
        pkgs = (import nixpkgs) { inherit system; };
        naersk' = pkgs.callPackage naersk { };
      in
      {
        devShell = pkgs.mkShell {
          # For `nix develop`
          nativeBuildInputs = with pkgs; [
            # Rust stuff - use unwrapped to avoid -Z flags
            rustfmt
            pre-commit
            rustPackages.clippy
            cargo-insta
            cargo-audit

            # Other
            openssl
            pkg-config
            chromium
          ];
        };

        # For `nix build`/`nix run`
        packages.default = naersk'.buildPackage { src = ./.; };
      }
    );
}
