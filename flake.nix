{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    crane.url = "github:ipetkov/crane";
    home-manager = {
      url = "github:nix-community/home-manager";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs =
    {
      nixpkgs,
      crane,
      systems,
      ...
    }@inputs:
    let
      inherit (nixpkgs) lib;
      forAllSystems = lib.genAttrs (import systems);
      pkgsFor = forAllSystems (system: nixpkgs.legacyPackages.${system});
    in
    {
      packages = forAllSystems (
        system:
        let
          pkgs = pkgsFor.${system};
          azalea = pkgs.callPackage ./nix/package.nix { craneLib = crane.mkLib pkgs; };
        in
        {
          default = azalea.azalea-pkg;
          inherit (azalea) azalea-pkg azalea-docs;

          test = pkgs.callPackage ./nix/test {
            azalea = azalea.azalea-pkg;
            inherit inputs;
          };

          docs = pkgs.linkFarm "azalea-desktop-shell-docs" [
            {
              name = "docs";
              path = "${azalea.azalea-docs}/share/doc";
            }
          ];
        }
      );

      devShells = forAllSystems (
        system:
        let
          pkgs = pkgsFor.${system};
        in
        {
          default = pkgs.mkShell {
            name = "azalea-devshell";
            RUST_SRC_PATH = "${pkgs.rust.packages.stable.rustPlatform.rustLibSrc}";
            G_MESSAGES_DEBUG = "Azalea";
            GDK_PIXBUF_MODULE_FILE = "${pkgs.gnome._gdkPixbufCacheBuilder_DO_NOT_USE {
              extraLoaders = with pkgs; [
                librsvg # icons
                webp-pixbuf-loader # webp
              ];
            }}";
            buildInputs = with pkgs; [
              nixfmt-tree

              cargo
              rustc
              rustfmt
              rust-analyzer
              pkg-config

              clang
              mold-wrapped

              gtk4
              gtk4-layer-shell
              openssl # reqwest
              dbus # bluer
              alsa-lib # volume

              zbus-xmlgen # generate zbus code from xml specs

              cloc
            ];
          };
        }
      );

      formatter = forAllSystems (system: pkgsFor.${system}.nixfmt-tree);
    };
}
