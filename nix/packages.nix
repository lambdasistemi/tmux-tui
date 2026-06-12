# Deliverables: the native `tmux-tui` CLI plus its static musl sibling.
# `meta.mainProgram` is required so the NixOS bundlers (AppImage/DEB/RPM) can
# resolve the executable.
{ craneEnv }:
let
  inherit (craneEnv) craneLib commonArgs cargoArtifacts muslArgs cargoArtifactsMusl;

  cli = craneLib.buildPackage (
    commonArgs
    // {
      inherit cargoArtifacts;
      meta.mainProgram = "tmux-tui";
    }
  );

  cli-musl = craneLib.buildPackage (
    muslArgs
    // {
      cargoArtifacts = cargoArtifactsMusl;
      meta.mainProgram = "tmux-tui";
    }
  );
in
{
  inherit cli cli-musl;
}
