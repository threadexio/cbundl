{
  description = "webpack but for C code.";

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixpkgs-unstable";
    flake-utils.url = "github:numtide/flake-utils";

    flake-compat = {
      url = "github:edolstra/flake-compat";
      flake = false;
    };

    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs = { nixpkgs, flake-utils, rust-overlay, ... }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = import nixpkgs {
          inherit system;
          overlays = [ (import rust-overlay) ];
        };

        rustVersion = pkgs.rust-bin.fromRustupToolchainFile ./rust-toolchain.toml;
        rustPlatform = pkgs.makeRustPlatform {
          cargo = rustVersion;
          rustc = rustVersion;
        };

        cargoToml = fromTOML (builtins.readFile ./Cargo.toml);

        cbundlDrv = rustPlatform.buildRustPackage {
          pname = cargoToml.package.name;
          version = cargoToml.package.version;
          src = ./.;

          nativeBuildInputs = with pkgs; [
            # Wanted by the build script.
            git
            nettools # only for hostname
          ];

          cargoLock.lockFile = ./Cargo.lock;

          meta = with pkgs.lib; {
            description = cargoToml.package.description;
            homepage = cargoToml.package.homepage;
            downloadPage = "${cargoToml.package.repository}/releases";
            license = licenses.asl20;
            mainProgram = cargoToml.package.name;
            platforms = platforms.all;
          };
        };
      in
      {
        formatter = pkgs.nixpkgs-fmt;

        devShells = rec {
          # For writing code.
          # $ nix develop
          dev = pkgs.mkShell {
            packages = [ rustVersion ];
          };

          # For editing the artwork.
          # $ nix develop '.#art'
          art = pkgs.mkShell {
            packages = with pkgs; [
              inkscape
              scour
            ];
          };

          default = dev;
        };

        packages.default = cbundlDrv;

        apps.default = {
          type = "app";
          program = "${cbundlDrv}/bin/cbundl";
        };
      }
    );
}
