{
  inputs.nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";

  outputs =
    {
      self,
      nixpkgs,
      systems,
      ...
    }:
    let
      inherit (nixpkgs) lib;
      forEachSystem = lib.genAttrs (import systems);
      pkgsFor = forEachSystem (system: nixpkgs.legacyPackages.${system});
      name = "azalea-desktop-shell";
    in
    {
      packages = forEachSystem (
        system:
        let
          pkgs = pkgsFor.${system};
          azalea = self.packages.${system}.azalea;
        in
        {
          default = self.packages.${system}.azalea;
          azalea = pkgs.callPackage ./nix/package.nix { };
        }
      );

      devShells = forEachSystem (
        system:
        let
          pkgs = pkgsFor.${system};
        in
        {
          default = pkgs.mkShell {
            name = "azalea-devshell";
            RUST_SRC_PATH = "${pkgs.rust.packages.stable.rustPlatform.rustLibSrc}";
            buildInputs = with pkgs; [
              cargo
              rustc
              rustfmt
              rust-analyzer
              pkg-config

              clang
              mold-wrapped

              gtk4
              gtk4-layer-shell

              cloc
            ];
          };
        }
      );
    };
}
