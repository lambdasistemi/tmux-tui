# Reproducible rustdoc HTML for the crate, co-hosted under the docs site
# at `/api/`. Unlike the `doc` check (which denies warnings as the
# correctness gate), this publish build renders even with doc warnings.
{ craneEnv }:
let
  inherit (craneEnv) craneLib commonArgs cargoArtifacts;
in
craneLib.cargoDoc (
  commonArgs
  // {
    inherit cargoArtifacts;
    pname = "tmux-tui-api-docs";
    cargoDocExtraArgs = "--no-deps --document-private-items";
  }
)
