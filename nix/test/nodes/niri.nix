{ pkgs, ... }:
let
  cfg = pkgs.writeText "azalea-niri.kdl" ''
    spawn-at-startup "foot"
    spawn-at-startup "systemctl" "--user" "start" "azalea.service"
    output "Virtual-1" {
      mode "1920x1080"
      scale 1.0
    }
  '';
in
{
  programs.niri.enable = true;

  systemd.user.services.azalea.enable = true;

  programs.bash.loginShellInit = ''
    if [ "$(tty)" = "/dev/tty1" ]; then
      exec dbus-run-session -- niri --session --config ${cfg}
    fi
  '';
}
