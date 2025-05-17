{
  lib,
  azalea,
  testers,
  kitty,
}:
testers.runNixOSTest {
  name = "azalea-integration-test";

  interactive.nodes.hyprland = import ./nodes/hyprland.nix;
  interactive.nodes.sway = import ./nodes/sway.nix;

  defaults = {
    services.getty.autologinUser = "alice";

    users.users.alice = {
      isNormalUser = true;
      uid = 1000;
    };

    environment.systemPackages = [
      azalea
      kitty
    ];

    systemd.user.services.azalea = {
      enable = lib.mkDefault false;

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
      machine.wait_for_file("/run/user/1000/wayland-1")
      machine.wait_until_succeeds("pgrep azalea")
      machine.sleep(6)

      with subtest(f"{machine.name}: default"):
        machine.screenshot(f"{machine.name}-default")

      machine.shutdown()

    for machine in machines:
      with subtest(machine.name):
        test(machine)
  '';
}
