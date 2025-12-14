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
  name = "azalea-interactive-integration-test";

  nodes.hyprland = import ./nodes/hyprland.nix;
  nodes.miracle-wm = import ./nodes/miracle-wm.nix;
  nodes.niri = import ./nodes/niri.nix;
  nodes.sway = import ./nodes/sway.nix;
  nodes.wayfire = import ./nodes/wayfire.nix;

  defaults = config // {
    virtualisation.qemu.options = [
      "-device virtio-vga-gl"
      "-display gtk,gl=on,zoom-to-fit=off"
    ];
  };

  testScript = ''
    print("This should only be executed in interactive mode!")
  '';
}
