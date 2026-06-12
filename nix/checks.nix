# Flake checks: the single Nix-first gate. Each entry is a real sandboxed
# crane derivation, so `nix flake check` actually runs clippy, rustfmt,
# the test suite, cargo-deny, and rustdoc — not just "a script ran".
{ craneEnv }:
let
  inherit (craneEnv) craneLib commonArgs cargoArtifacts;
in
{
  clippy = craneLib.cargoClippy (
    commonArgs
    // {
      inherit cargoArtifacts;
      cargoClippyExtraArgs = "--all-targets --all-features -- --deny warnings";
    }
  );

  fmt = craneLib.cargoFmt { inherit (commonArgs) src; };

  nextest = craneLib.cargoNextest (
    commonArgs
    // {
      inherit cargoArtifacts;
    }
  );

  deny = craneLib.cargoDeny { inherit (commonArgs) src; };

  doc = craneLib.cargoDoc (
    commonArgs
    // {
      inherit cargoArtifacts;
      env.RUSTDOCFLAGS = "--deny warnings";
    }
  );
}
