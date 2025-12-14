{
  inputs,
  lib,
  azalea,
  foot,
  testers,
}:
let
  user = "alice";
  config = import ./config.nix {
    inherit
      user
      inputs
      lib
      azalea
      foot
      ;
  };
in
testers.runNixOSTest {
  name = "azalea-integration-test";

  nodes.hyprland = import ./nodes/hyprland.nix;
  nodes.miracle-wm = import ./nodes/miracle-wm.nix;
  # nodes.niri = import ./nodes/niri.nix; # Niri doesn't support software rendering
  nodes.sway = import ./nodes/sway.nix;
  nodes.wayfire = import ./nodes/wayfire.nix;

  defaults = config // {
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
