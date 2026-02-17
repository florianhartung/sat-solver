{
  description = "a basic SAT solver";

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixos-unstable";

    devshell.url = "github:numtide/devshell";
    utils.url = "github:numtide/flake-utils";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    treefmt-nix = {
      url = "github:numtide/treefmt-nix";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };
  outputs =
    {
      self,
      nixpkgs,
      devshell,
      utils,
      rust-overlay,
      treefmt-nix,
    }:
    utils.lib.eachDefaultSystem (
      system:
      let
        overlays = [
          devshell.overlays.default
          (import rust-overlay)
        ];
        pkgs = import nixpkgs {
          inherit system overlays;
        };
        rust-target = pkgs.pkgsStatic.stdenv.targetPlatform.rust.rustcTarget;
        rust-toolchain = pkgs.rust-bin.stable.latest.default.override {
          extensions = [ "rust-src" ];
          targets = [
            rust-target
          ];
        };
        treefmtEval = treefmt-nix.lib.evalModule pkgs ./treefmt.nix;

      in
      {
        git.hooks = {
          enable = true;
          pre-commit.text = "nix flake check";
        };

        devShells.default = (
          pkgs.devshell.mkShell {
            name = "dev";
            packages = with pkgs; [
              stdenv.cc
              coreutils

              rust-toolchain
              rust-analyzer
              cargo-flamegraph
            ];
          }
        );

        formatter = treefmtEval.config.build.wrapper;

        checks = {
          formatting = treefmtEval.config.build.check self;
        };
      }
    );
}
