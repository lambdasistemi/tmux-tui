{
  description = "tmux-tui: a mouse-driven drag-and-drop layout manager for tmux";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    crane.url = "github:ipetkov/crane";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs =
    { nixpkgs
    , flake-utils
    , crane
    , rust-overlay
    , ...
    }:
    flake-utils.lib.eachDefaultSystem (
      system:
      let
        pkgs = import nixpkgs {
          inherit system;
          overlays = [ (import rust-overlay) ];
        };

        rustToolchain = import ./nix/toolchain.nix { inherit pkgs; };

        craneEnv = import ./nix/crane.nix {
          inherit pkgs crane rustToolchain;
          src = ./.;
        };

        packages = import ./nix/packages.nix { inherit craneEnv; };
        checks = import ./nix/checks.nix { inherit craneEnv; };
        apps = import ./nix/apps.nix { inherit pkgs; };
        api-docs = import ./nix/api-docs.nix { inherit craneEnv; };
      in
      {
        packages = {
          default = packages.cli;
          inherit (packages) cli;
          inherit api-docs;
          release-artifacts = import ./nix/release.nix { inherit pkgs packages; };
        };

        inherit checks apps;

        devShells.default = craneEnv.craneLib.devShell {
          packages = [
            pkgs.just
            pkgs.cargo-deny
          ];
        };
      }
    );
}
