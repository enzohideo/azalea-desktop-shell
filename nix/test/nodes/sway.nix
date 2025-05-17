{ ... }:
{
  programs.sway.enable = true;

  programs.bash.loginShellInit = ''
    mkdir -p ~/.config/sway
    sed s/foot/kitty/ /etc/sway/config > ~/.config/sway/config
    exec sway
  '';
}
