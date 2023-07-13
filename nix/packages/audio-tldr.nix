{
  rustPlatform,
  lib,
}: let
  src = builtins.path {
    name = "audio-tldr";
    path = lib.cleanSource ../..;
  };
  config = lib.trivial.importTOML ../../Cargo.toml;
in
  rustPlatform.buildRustPackage {
    pname = config.package.name;
    version = config.package.version;

    inherit src;

    cargoDeps = {
      lockFile = "../../Cargo.lock";
    };

    cargoLock = {
      lockFile = ../../Cargo.lock;
    };

    meta = with lib; {
      description = "Audio TL;DR generator";
      # homepage = "";
      license = licenses.mit;
      maintainers = [maintainers.MalteT];
    };
  }
