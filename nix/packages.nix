# The deliverable: the native `tmux-tui` CLI binary.
{ craneEnv }:
let
  inherit (craneEnv) craneLib commonArgs cargoArtifacts;

  cli = craneLib.buildPackage (
    commonArgs
    // {
      inherit cargoArtifacts;
    }
  );
in
{
  inherit cli;
}
