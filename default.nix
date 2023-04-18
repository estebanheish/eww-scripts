{
  rustPlatform,
  pkg-config,
  alsa-lib,
  alsa-utils,
}:
rustPlatform.buildRustPackage rec {
  pname = "eww-scripts";
  version = "0.1.0";

  src = ./.;

  cargoLock = {
    lockFile = ./Cargo.lock;
  };

  nativeBuildInputs = [
    pkg-config
  ];

  buildInputs = [alsa-utils alsa-lib];
}
