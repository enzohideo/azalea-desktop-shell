{
  lib,
  nix-gitignore,
  rustPlatform,
  pkg-config,
  wrapGAppsHook4,
  gtk4,
  gtk4-layer-shell,
  clang,
  mold-wrapped,
}:

rustPlatform.buildRustPackage (finalAttrs: {
  pname = "azalea";
  version = "v0.0.0";

  useFetchCargoVendor = true;
  cargoHash = "sha256-OdWVBjiW91+9UpuQHmp8Ut5gp7prBdsW37UizJBZOfI=";

  src = nix-gitignore.gitignoreSource [
    "flake.*\n"
    "nix\n"
  ] (lib.cleanSource ../.);

  NIX_OUTPATH_USED_AS_RANDOM_SEED = "aaaaaaaaaa";

  nativeBuildInputs = [
    rustPlatform.bindgenHook
    pkg-config
    wrapGAppsHook4
    clang
    mold-wrapped
  ];

  buildInputs = [
    gtk4
    gtk4-layer-shell
  ];

  meta = with lib; {
    mainProgram = finalAttrs.pname;
    maintainers = with maintainers; [ enzohideo ];
  };
})
