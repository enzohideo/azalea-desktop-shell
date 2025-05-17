{ ... }:
{
  programs.hyprland = {
    enable = true;
    withUWSM = true;
  };

  programs.bash.loginShellInit = ''
    if uwsm check may-start; then
      exec uwsm start hyprland-uwsm.desktop
    fi
  '';
}
