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

  # TODO: Build package with all screenshots, possibly RunCommand if necessary
  testScript = ''
    def test(machine):
      machine.start()
      machine.wait_for_unit("multi-user.target")
      machine.wait_until_succeeds("pgrep -x '.azalea-wrapped'")
      machine.sleep(3)

      with subtest(f"{machine.name}: default"):
        machine.screenshot(f"{machine.name}-default")

      machine.shutdown()

    for machine in machines:
      with subtest(machine.name):
        test(machine)
  '';
}
