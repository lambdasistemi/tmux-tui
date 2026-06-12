# NixOS module for tmux-tui.
#
# This installs the binary system-wide. The popup keybindings live in a user's
# tmux config, so wire those with the Home Manager module (or your own
# programs.tmux.extraConfig) rather than here.
{ self }:
{ config, lib, pkgs, ... }:
let
  cfg = config.programs.tmux-tui;
in
{
  options.programs.tmux-tui = {
    enable = lib.mkEnableOption "tmux-tui, a mouse-driven drag-and-drop layout manager for tmux (system-wide install)";

    package = lib.mkOption {
      type = lib.types.package;
      default = self.packages.${pkgs.stdenv.hostPlatform.system}.default;
      defaultText = lib.literalExpression "tmux-tui.packages.\${system}.default";
      description = "The tmux-tui package to install system-wide.";
    };
  };

  config = lib.mkIf cfg.enable {
    environment.systemPackages = [ cfg.package ];
  };
}
