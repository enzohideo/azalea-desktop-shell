{ pkgs, ... }:
let
  cfg = pkgs.writeText "azalea-sway.conf" ''
    include /etc/sway/config.d/*

    output * bg /run/current-system/sw/share/backgrounds/sway/Sway_Wallpaper_Blue_1920x1080.png fill
    for_window [class="^.*"] opacity 0.7

    exec foot
  '';
in
{
  programs.sway.enable = true;

  systemd.user.services.azalea.enable = true;

  programs.bash.loginShellInit = ''
    if [ "$(tty)" = "/dev/tty1" ]; then
      exec dbus-run-session -- sway --config ${cfg}
    fi
  '';
}
