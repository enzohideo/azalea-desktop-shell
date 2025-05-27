{
  lib,
  azalea,
  testers,
  foot,
}:
testers.runNixOSTest {
  name = "azalea-integration-test";

  nodes.hyprland = import ./nodes/hyprland.nix;
  nodes.sway = import ./nodes/sway.nix;

  defaults = {
    services.getty.autologinUser = "alice";

    users.users.alice = {
      isNormalUser = true;
      uid = 1000;
    };

    environment.systemPackages = [
      azalea
      foot
    ];

    systemd.user.services.azalea = {
      enable = lib.mkDefault false;

      description = "Azalea Daemon";

      after = [ "graphical-session.target" ];
      wantedBy = [ "graphical-session.target" ];
      bindsTo = [ "graphical-session.target" ];

      unitConfig = {
        ConditionEnvironment = "WAYLAND_DISPLAY";
      };

      serviceConfig = {
        ExecStart = "${azalea}/bin/azalea daemon start";
        Restart = "on-failure";
      };
    };

    virtualisation.memorySize = 8192;
    virtualisation.writableStore = true;
  };

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
