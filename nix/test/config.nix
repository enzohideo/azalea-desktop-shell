{
  inputs,
  lib,
  azalea,
  user,
  foot,
}:
{
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

  systemd.user.services =
    let
      service = {
        enable = lib.mkDefault false;

        description = "Azalea Daemon";

        after = [ "graphical-session.target" ];
        wantedBy = [ "graphical-session.target" ];
        bindsTo = [ "graphical-session.target" ];

        unitConfig = {
          ConditionEnvironment = "WAYLAND_DISPLAY";
        };
      };
    in
    {
      azalea = service // {
        serviceConfig.ExecStart = "${azalea}/bin/azalea daemon start --config ${./config.ron}";
      };
      azalea-default = service // {
        serviceConfig.ExecStart = "${azalea}/bin/azalea daemon start";
      };
    };

  virtualisation.memorySize = 8192;
  virtualisation.writableStore = true;

}
