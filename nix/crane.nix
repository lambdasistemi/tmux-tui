# Builds the crane library bound to the pinned toolchain and the shared
# argument records used by both packages and checks. A single
# dependency-only artifact set warms the store once; the CLI build,
# clippy, fmt, nextest, deny, and doc all reuse it.
{ pkgs
, crane
, rustToolchain
, src
}:
let
  craneLib = (crane.mkLib pkgs).overrideToolchain rustToolchain;

  commonArgs = {
    src = craneLib.cleanCargoSource src;
    strictDeps = true;
    pname = "tmux-tui";
    version = "0.1.0";

    buildInputs = pkgs.lib.optionals pkgs.stdenv.isDarwin [
      pkgs.libiconv
    ];
  };

  cargoArtifacts = craneLib.buildDepsOnly commonArgs;
in
{
  inherit craneLib commonArgs cargoArtifacts;
}
