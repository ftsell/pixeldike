{
  description = "pixelflut - a pixel drawing game for programmers";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs = { self, nixpkgs, flake-utils, rust-overlay }: flake-utils.lib.eachDefaultSystem (system:
    let
      pkgs = import nixpkgs {
        system = system;
        overlays = [ (import rust-overlay) ];
      };
      rustPlatform = pkgs.makeRustPlatform rec {
        cargo = (pkgs.rust-bin.fromRustupToolchainFile ./rust-toolchain.toml);
        rustc = cargo;
      };
      cargoToml = (builtins.fromTOML (builtins.readFile ./Cargo.toml));
    in
    rec {
      packages = rec {
        default = pixeldike;
        pixeldike = rustPlatform.buildRustPackage {
          name = "pixeldike";
          version = cargoToml.package.version;
          src = ./.;
          cargoLock.lockFile = ./Cargo.lock;
          buildFeatures = [ "default" "ws" "windowing" ];
          RUSTFLAGS = "--cfg tokio_unstable ";
        };
        pixeldike-oci = pkgs.dockerTools.buildLayeredImage {
          name = "ghcr.io/ftsell/pixeldike";
          tag = "latest";
          config = {
            Cmd = [ "${pixeldike}/bin/pixeldike" ];
            User = "1000:1000";
          };
        };
      };

      apps = rec {
        default = pixeldike;
        pixeldike = flake-utils.lib.mkApp {
          drv = packages.pixeldike;
        };
      };

      devShells.default = pkgs.mkShell {
        packages = with pkgs; [
          rustPlatform.rust.cargo
          pre-commit
        ];
      };
    });
}
