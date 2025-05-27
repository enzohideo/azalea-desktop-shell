{ pkgs, ... }:
let
  cfg = pkgs.writeText "azalea-hyprland.conf" ''
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
      exec dbus-run-session -- hyprland --config ${cfg}
    fi
  '';
}
