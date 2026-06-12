# Home Manager module for tmux-tui.
#
# tmux configuration is per-user, so this is the natural home for the popup
# keybindings. Importing the module installs nothing surprising: the package is
# only installed when `enable = true`, and the tmux keybindings are off until
# `keybindings = true` so an existing config is never clobbered.
{ self }:
{ config, lib, pkgs, ... }:
let
  cfg = config.programs.tmux-tui;
  popup = "display-popup -E -w ${cfg.size} -h ${cfg.size} tmux-tui";
in
{
  options.programs.tmux-tui = {
    enable = lib.mkEnableOption "tmux-tui, a mouse-driven drag-and-drop layout manager for tmux";

    package = lib.mkOption {
      type = lib.types.package;
      default = self.packages.${pkgs.stdenv.hostPlatform.system}.default;
      defaultText = lib.literalExpression "tmux-tui.packages.\${system}.default";
      description = "The tmux-tui package to install.";
    };

    keybindings = lib.mkEnableOption "tmux keybindings that launch the tmux-tui popup (requires programs.tmux.enable)";

    bindKey = lib.mkOption {
      type = lib.types.str;
      default = "g";
      example = "T";
      description = "Prefix key bound to the tmux-tui popup, when keybindings are enabled.";
    };

    mouse = lib.mkOption {
      type = lib.types.bool;
      default = true;
      description = "Also bind Ctrl+right-click on a pane to the tmux-tui popup, when keybindings are enabled.";
    };

    size = lib.mkOption {
      type = lib.types.str;
      default = "30%";
      example = "50%";
      description = "Popup width and height, passed to tmux `display-popup -w/-h`.";
    };
  };

  config = lib.mkIf cfg.enable {
    home.packages = [ cfg.package ];

    assertions = [
      {
        assertion = !cfg.keybindings || config.programs.tmux.enable;
        message = "programs.tmux-tui.keybindings requires programs.tmux.enable = true.";
      }
    ];

    programs.tmux.extraConfig = lib.mkIf cfg.keybindings (
      "bind-key ${cfg.bindKey} ${popup}\n"
      + lib.optionalString cfg.mouse "bind-key -n C-MouseDown3Pane ${popup}\n"
    );
  };
}
