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
    # Release-artifact tooling: NixOS bundlers (AppImage/DEB/RPM) and the
    # shared lambdasistemi artifact lib (Linux bundle + musl tarball + Homebrew).
    bundlers = {
      url = "github:NixOS/bundlers";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    dev-assets.url = "github:paolino/dev-assets/v0.1.0";
  };

  outputs =
    { self
    , nixpkgs
    , flake-utils
    , crane
    , rust-overlay
    , bundlers
    , dev-assets
    , ...
    }:
    flake-utils.lib.eachDefaultSystem (
      system:
      let
        pkgs = import nixpkgs {
          inherit system;
          overlays = [ (import rust-overlay) ];
        };

        version = "0.1.0";

        rustToolchain = import ./nix/toolchain.nix { inherit pkgs; };

        craneEnv = import ./nix/crane.nix {
          inherit pkgs crane rustToolchain;
          src = ./.;
        };

        packages = import ./nix/packages.nix { inherit craneEnv; };
        checks = import ./nix/checks.nix { inherit craneEnv; };
        apps = import ./nix/apps.nix { inherit pkgs; };
        api-docs = import ./nix/api-docs.nix { inherit craneEnv; };

        # Per-arch Linux artifacts (AppImage/DEB/RPM + static musl tarball),
        # built natively on each arch's runner. Forced only on Linux.
        linuxArtifacts = dev-assets.lib.mkLinuxArtifacts {
          inherit pkgs system version;
          executableName = "tmux-tui";
          glibcPackage = packages.cli;
          muslPackage = packages.cli-musl;
          inherit bundlers;
        };

        # Dev (PR) artifacts. The dev-assets linux-release action names the dev
        # build `<version>-<short-sha>` and the smoke does an exact `test -f`,
        # so the artifact filenames MUST carry the same suffix or the smoke
        # fails silently. self.shortRev matches the action's
        # `git rev-parse --short=7 HEAD` on a clean CI checkout.
        devVersion = "${version}-${self.shortRev or (self.dirtyShortRev or "dirty")}";
        linuxDevArtifacts = dev-assets.lib.mkLinuxArtifacts {
          inherit pkgs system version;
          artifactVersion = devVersion;
          executableName = "tmux-tui";
          glibcPackage = packages.cli;
          muslPackage = packages.cli-musl;
          inherit bundlers;
        };

        # macOS tarball + Homebrew formula, built natively on a macOS runner.
        # Forced only on Darwin (the lib asserts isDarwin).
        mkDarwin = extra: dev-assets.lib.mkDarwinHomebrewBundle { inherit pkgs; } ({
          pname = "tmux-tui";
          inherit version;
          owner = "lambdasistemi";
          desc = "Mouse-driven drag-and-drop layout manager for tmux";
          formulaClass = "TmuxTui";
          executables = { tmux-tui = packages.cli; };
          # The default smoke runs `tmux-tui --help`, but tmux-tui takes no flags
          # and exits non-zero off-tmux, which would fail the bundle's `set -e`.
          # Run it with no args, tolerate the exit, and grep the known message.
          smokeCommands = [
            "tmux-tui >/tmp/tmux-tui-smoke.out 2>&1 || true"
            ''grep -F -- "not inside a tmux session" /tmp/tmux-tui-smoke.out''
          ];
        } // extra);

        darwinArtifacts = mkDarwin { };

        # Dev (PR) bundle: its own formula name/class + rev-suffixed version, so
        # the dev-homebrew action finds `tmux-tui-dev.rb`.
        darwinDevArtifacts = mkDarwin {
          artifactVersion = devVersion;
          releaseTag = "dev-homebrew";
          formulaName = "tmux-tui-dev";
          formulaClass = "TmuxTuiDev";
          formulaVersion = devVersion;
        };
      in
      {
        packages = {
          default = packages.cli;
          inherit (packages) cli;
          inherit api-docs;
        }
        // pkgs.lib.optionalAttrs pkgs.stdenv.isLinux {
          inherit (packages) cli-musl;
          tmux-tui-linux-release-artifacts = linuxArtifacts;
          tmux-tui-linux-dev-release-artifacts = linuxDevArtifacts;
          linux-artifact-smoke = dev-assets.lib.mkLinuxArtifactSmoke { inherit pkgs system; };
        }
        // pkgs.lib.optionalAttrs pkgs.stdenv.isDarwin {
          tmux-tui-darwin-release-artifacts = darwinArtifacts;
          tmux-tui-darwin-dev-homebrew-artifacts = darwinDevArtifacts;
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
