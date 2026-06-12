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

  # Statically-linked musl build (Linux only). tmux-tui is pure Rust (ratatui +
  # crossterm, no C deps), so the musl target links a fully static binary with
  # no C cross-toolchain. Evaluated lazily; only forced on Linux.
  muslTarget =
    if pkgs.stdenv.hostPlatform.isAarch64
    then "aarch64-unknown-linux-musl"
    else "x86_64-unknown-linux-musl";
  muslArgs = commonArgs // {
    CARGO_BUILD_TARGET = muslTarget;
    CARGO_BUILD_RUSTFLAGS = "-C target-feature=+crt-static";
    doCheck = false;
  };
  cargoArtifactsMusl = craneLib.buildDepsOnly muslArgs;
in
{
  inherit craneLib commonArgs cargoArtifacts muslArgs cargoArtifactsMusl;
}
