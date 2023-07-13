{
  inputs.devshell.url = "github:numtide/devshell";
  inputs.flake-parts.url = "github:hercules-ci/flake-parts";
  inputs.treefmt-nix.url = "github:numtide/treefmt-nix";
  inputs.pre-commit-hooks-nix.url = "github:cachix/pre-commit-hooks.nix";
  inputs.nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";

  outputs = inputs @ {
    flake-parts,
    devshell,
    treefmt-nix,
    pre-commit-hooks-nix,
    ...
  }:
    flake-parts.lib.mkFlake {inherit inputs;} {
      imports = [
        devshell.flakeModule
        treefmt-nix.flakeModule
        pre-commit-hooks-nix.flakeModule
      ];
      systems = [
        "x86_64-linux"
      ];
      perSystem = {
        self',
        pkgs,
        config,
        ...
      }: {
        pre-commit.check.enable = true;
        pre-commit.settings.hooks.treefmt.enable = true;
        # Main package
        packages.audio-tldr = pkgs.callPackage ./nix/packages/audio-tldr.nix {};
        packages.default = self'.packages.audio-tldr;
        # Shell
        devShells.default = pkgs.stdenv.mkDerivation {
          name = "audio-tldr";
          nativeBuildInputs = [
            config.treefmt.package
            pkgs.cargo
            pkgs.clippy
            pkgs.rustfmt
            pkgs.nil
            pkgs.pkg-config
            pkgs.rust-analyzer
            pkgs.rustc
          ];
          RUST_LOG = "trace";
        };
        devShells.pre-commit = config.pre-commit.devShell;
        # Formatter
        treefmt.projectRootFile = "flake.nix";
        treefmt.programs = {
          rustfmt.enable = true;
          alejandra.enable = true;
        };
      };
    };
}
