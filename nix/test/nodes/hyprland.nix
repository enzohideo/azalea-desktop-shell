{ pkgs, ... }:
let
  cfg = pkgs.writeText "azalea-hyprland.conf" ''
    monitor=,1920x1080,auto,1

    exec-once = systemctl --user start azalea.service
    exec-once = foot

    decoration {
      blur {
        enabled = false
      }
      active_opacity = 0.70
      inactive_opacity = 0.70
      fullscreen_opacity = 0.70
    }
  '';
in
{
  programs.hyprland.enable = true;

  systemd.user.services.azalea.enable = true;

  programs.bash.loginShellInit = ''
    if [ "$(tty)" = "/dev/tty1" ]; then
      exec hyprland --config ${cfg}
    fi
  '';
}
