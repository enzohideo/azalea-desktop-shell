{ pkgs, ... }:
let
  # FIXME: Transparent kitty background
  cfg = pkgs.writeText "azalea-sway.conf" ''
    include /etc/sway/config.d/*
    exec kitty
  '';
in
{
  programs.sway.enable = true;

  systemd.user.services.azalea.enable = true;

  environment.systemPackages = [
    pkgs.swaybg
  ];

  programs.bash.loginShellInit = ''
    if [ "$(tty)" = "/dev/tty1" ]; then
      exec dbus-run-session -- sway --config ${cfg}
    fi
  '';
}
