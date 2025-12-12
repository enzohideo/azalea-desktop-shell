{ pkgs, ... }:
let
  cfg = pkgs.writeText "azalea-wayfire.ini" ''
    [autostart]
    azalea = systemctl --user import-environment WAYLAND_DISPLAY && systemctl --user start azalea
    autostart_wf_shell = false
    background = wf-background

    [output:Virtual-1]
    mode = 1920x1080
    scale = 1
  '';
in
{
  programs.wayfire.enable = true;

  systemd.user.services.azalea.enable = true;

  programs.bash.loginShellInit = ''
    if [ "$(tty)" = "/dev/tty1" ]; then
      exec dbus-run-session -- wayfire --config ${cfg}
    fi
  '';
}
