{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    crane.url = "github:ipetkov/crane";
  };

  outputs =
    {
      nixpkgs,
      crane,
      systems,
      ...
    }:
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
          default = azalea;
          inherit azalea;

          test = pkgsFor.${system}.callPackage ./nix/test {
            inherit azalea;
          };
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
            G_MESSAGES_DEBUG = "Azalea";
            RUST_SRC_PATH = "${pkgs.rust.packages.stable.rustPlatform.rustLibSrc}";
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

              cloc
            ];
          };
        }
      );

      formatter = forAllSystems (system: pkgsFor.${system}.nixfmt-tree);
    };
}
