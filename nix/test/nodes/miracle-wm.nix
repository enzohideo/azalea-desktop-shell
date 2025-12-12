{ pkgs, ... }:
let
  cfg = pkgs.writeText "azalea-miracle-wm-display.yaml" ''
    outputs:
      - enabled: true
        name: Virtual-1
        position:
          x: 0
          y: 0
        size:
          width: 1920
          height: 1080
        refresh: 60
        orientation: normal
        scale: 1
        group_id: 0
  '';
in
{
  programs.wayland.miracle-wm.enable = true;

  systemd.user.services.azalea.enable = true;

  programs.bash.loginShellInit = ''
    if [ "$(tty)" = "/dev/tty1" ]; then
      mkdir -p "$HOME"/.config/miracle-wm
      cp ${cfg} "$HOME"/.config/miracle-wm/display.yaml
      exec miracle-wm-session
    fi
  '';
}
