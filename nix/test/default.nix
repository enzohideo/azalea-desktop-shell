{
  testers,
  azalea,
}:
testers.runNixOSTest {
  name = "azalea-integration-test";

  interactive.nodes.hyprland = import ./nodes/hyprland.nix;

  defaults = {
    services.getty.autologinUser = "alice";

    users.users.alice = {
      isNormalUser = true;
      uid = 1000;
    };

    environment.systemPackages = [
      azalea
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
    hyprland.start()
    hyprland.wait_for_unit("multi-user.target")
    hyprland.wait_until_succeeds("pgrep -x '.azalea-wrapped'")
    hyprland.sleep(3)
    hyprland.screenshot("hyprland-default")
    hyprland.shutdown()
  '';
}
