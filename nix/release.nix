# Tag-driven release bundle: the native CLI tarball plus a SHA256SUMS
# manifest. Nix-built from the committed Cargo.lock with no network, so
# the bundle is reproducible.
{ pkgs, packages }:
let
  inherit (packages) cli;
  version = "0.1.0";
  inherit (pkgs.stdenv.hostPlatform) system;
in
pkgs.runCommand "tmux-tui-release-artifacts"
{
  nativeBuildInputs = [
    pkgs.gnutar
    pkgs.gzip
    pkgs.coreutils
  ];
}
  ''
    set -euo pipefail
    mkdir -p "$out"
    tar -czf "$out/tmux-tui-${version}-${system}.tar.gz" -C ${cli}/bin tmux-tui
    ( cd "$out" && sha256sum -- * > SHA256SUMS )
  ''
