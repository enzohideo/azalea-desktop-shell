{
  testers,
  kitty,
  azalea,
}:
testers.runNixOSTest {
  name = "azalea-hyprland";

  nodes.machine =
    { ... }:
    {
      services.getty.autologinUser = "alice";

      users.users.alice = {
        isNormalUser = true;
        uid = 1000;
      };

      programs.hyprland = {
        enable = true;
        withUWSM = true;
      };

      programs.bash.loginShellInit = ''
        if [ "$(tty)" = "/dev/tty1" ]; then
          if uwsm check may-start; then
              exec uwsm start hyprland-uwsm.desktop
          fi
        fi
      '';

      environment.systemPackages = [
        azalea
        kitty
      ];

      systemd.user.services.azalea = {
        enable = true;

        description = "Azalea Daemon";
        after = [ "graphical-session.target" ];
        wantedBy = [ "graphical-session.target" ];
        unitConfig = {
          ConditionEnvironment = "WAYLAND_DISPLAY";
        };
        serviceConfig = {
          ExecStart = "${azalea}/bin/azalea daemon start";
          Restart = "on-failure";
        };
      };
    };

  # TODO: Automate this, possibly RunCommand if necessary
  testScript = ''
    start_all()
    machine.wait_for_unit("multi-user.target")
    machine.wait_until_succeeds("pgrep -x '.azalea-wrapped'")
    machine.sleep(3)
    machine.screenshot("hyprland-default")
  '';
}
