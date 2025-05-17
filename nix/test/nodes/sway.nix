{ pkgs, ... }:
let
  # FIXME: Transparent kitty background
  cfg = pkgs.writeText "azalea-sway.conf" ''
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
      exec sway --config ${cfg}
    fi
  '';
}
