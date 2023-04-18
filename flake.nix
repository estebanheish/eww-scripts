{
  description = "A very basic flake";

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixos-unstable";
  };

  outputs = {
    self,
    nixpkgs,
  }: let
    allSystems = [
      "x86_64-linux" # AMD/Intel Linux
      "aarch64-linux" # ARM Linux
    ];

    forAllSystems = fn:
      nixpkgs.lib.genAttrs allSystems
      (system:
        fn {
          pkgs = import nixpkgs {inherit system;};
        });
  in {
    devShells = forAllSystems ({pkgs}: {
      default = pkgs.mkShell {
        buildInputs = with pkgs; [
          pkg-config
          alsa-lib
          alsa-utils
        ];
      };
    });

    packages = forAllSystems ({pkgs}: rec {
      default = pkgs.callPackage ./default.nix {};
    });
  };
}
