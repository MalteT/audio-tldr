{
  rustPlatform,
  lib,
  pkg-config,
  openssl,
}: let
  src = builtins.path {
    name = "audio-tldr-source";
    path = lib.sources.sourceByRegex ../.. [
      ".*Cargo\.toml"
      ".*Cargo\.lock"
      ".*src.*"
      ".*\.rs"
    ];
  };
  config = lib.trivial.importTOML ../../Cargo.toml;
in
  rustPlatform.buildRustPackage {
    pname = config.package.name;
    version = config.package.version;

    inherit src;

    nativeBuildInputs = [
      pkg-config
    ];

    propagatedBuildInputs = [
      openssl
    ];

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
