{
  lib,
  craneLib,
  rustPlatform,
  pkg-config,
  wrapGAppsHook4,
  gnome,
  librsvg,
  gtk4,
  gtk4-layer-shell,
  openssl,
  clang,
  mold-wrapped,
}:

let
  src = craneLib.cleanCargoSource ../.;

  commonArgs = {
    inherit src;
    strictDeps = true;

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
      openssl
    ];

    NIX_OUTPATH_USED_AS_RANDOM_SEED = "aaaaaaaaaa";
  };

  cargoArtifacts = craneLib.buildDepsOnly commonArgs;

  individualCrateArgs = commonArgs // {
    inherit cargoArtifacts;
    inherit (craneLib.crateNameFromCargoToml { inherit src; }) version;
    doCheck = false;
  };

  fileSetForCrate =
    crate:
    lib.fileset.toSource {
      root = ../.;
      fileset = lib.fileset.unions [
        ../Cargo.toml
        ../Cargo.lock
        (craneLib.fileset.commonCargoSources ../azalea-core)
        (craneLib.fileset.commonCargoSources ../azalea-derive)
        (craneLib.fileset.commonCargoSources ../azalea-log)
        (craneLib.fileset.commonCargoSources ../azalea-service)
        (craneLib.fileset.commonCargoSources ../azalea-shell)
        (craneLib.fileset.commonCargoSources crate)
      ];
    };
in
craneLib.buildPackage (
  individualCrateArgs
  // rec {
    pname = "azalea";
    cargoExtraArgs = "-p azalea";
    src = fileSetForCrate ../azalea;

    postInstall = ''
      export GDK_PIXBUF_MODULE_FILE="${
        gnome._gdkPixbufCacheBuilder_DO_NOT_USE {
          extraLoaders = [
            librsvg
          ];
        }
      }";
    '';

    meta = with lib; {
      mainProgram = pname;
      maintainers = with maintainers; [ enzohideo ];
    };
  }
)
