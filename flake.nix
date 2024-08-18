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

  outputs = { self, nixpkgs, flake-utils, rust-overlay }: rec {
      packages.x86_64-linux = let 
        pkgs = import nixpkgs {
          system = "x86_64-linux";
          overlays = [ (import rust-overlay) ];
        };
        rustPlatform = pkgs.makeRustPlatform rec {
          cargo = (pkgs.rust-bin.fromRustupToolchainFile ./rust-toolchain.toml);
          rustc = cargo;
        };
        cargoToml = (builtins.fromTOML (builtins.readFile ./Cargo.toml));
      in rec {
        default = pixeldike;
        pixeldike = rustPlatform.buildRustPackage {
          name = "pixeldike";
          version = cargoToml.package.version;
          src = ./.;
          cargoLock.lockFile = ./Cargo.lock;
          buildFeatures = [ "default" "ws" "windowing" ];
          RUSTFLAGS = "--cfg tokio_unstable ";
        };
      };

      apps.x86_64-linux = rec {
        default = pixeldike;
        pixeldike = flake-utils.lib.mkApp {
          drv = packages.x86_64-linux.pixeldike;
        };
      };
    };
}
