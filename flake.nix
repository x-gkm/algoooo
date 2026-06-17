{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs?ref=nixos-unstable";
    flake-parts = {
      url = "github:hercules-ci/flake-parts";
      inputs.nixpkgs-lib.follows = "nixpkgs";
    };
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    naersk = {
      url = "github:nix-community/naersk";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs =
    inputs:
    inputs.flake-parts.lib.mkFlake { inherit inputs; } {
      systems = [
        "x86_64-linux"
        "aarch64-linux"
        "x86_64-darwin"
        "aarch64-darwin"
      ];
      perSystem =
        {
          system,
          config,
          pkgs,
          inputs',
          ...
        }:
        let
          toolchain = pkgs.rust-bin.stable.latest.default.override {
            extensions = [ "rust-src" ];
          };

          naersk' = pkgs.callPackage inputs.naersk {
            rustc = toolchain;
            cargo = toolchain;
          };
        in
        {
          _module.args.pkgs = import inputs.nixpkgs {
            inherit system;
            overlays = [
              inputs.rust-overlay.overlays.default
            ];
          };

          packages.default = naersk'.buildPackage {
            src = ./.;
          };

          devShells.default =
            pkgs.mkShell {
              nativeBuildInputs = [
                toolchain
                pkgs.sqlx-cli
              ];
            };
        };
    };
}
