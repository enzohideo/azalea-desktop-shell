{
  inputs,
  lib,
  azalea,
  testers,
  foot,
}:
let
  user = "alice";
in
testers.runNixOSTest {
  name = "azalea-integration-test";

  nodes.hyprland = import ./nodes/hyprland.nix;
  nodes.miracle-wm = import ./nodes/miracle-wm.nix;
  # nodes.niri = import ./nodes/niri.nix; # Niri doesn't support software rendering
  nodes.sway = import ./nodes/sway.nix;
  nodes.wayfire = import ./nodes/wayfire.nix;

  defaults = {
    imports = [
      inputs.home-manager.nixosModules.home-manager
    ];

    services.getty.autologinUser = user;

    users.users.${user} = {
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
        ExecStart = "${azalea}/bin/azalea daemon start --config ${./config.ron}";
        Restart = "on-failure";
      };
    };

    virtualisation.memorySize = 8192;
    virtualisation.writableStore = true;
    virtualisation.qemu.options = [ "-vga virtio" ];
  };

  testScript = ''
    def test(machine):
      machine.start()

      machine.wait_for_unit("multi-user.target")
      machine.wait_until_succeeds("pgrep azalea")
      machine.sleep(10)

      with subtest(f"{machine.name}: default"):
        machine.screenshot(f"{machine.name}-default")

      machine.shutdown()

    for machine in machines:
      with subtest(machine.name):
        test(machine)
  '';
}
