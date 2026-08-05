{
  description = "devshell flake for github:lavafroth/zilch";

  outputs =
    {
      nixpkgs,
      ...
    }:
    let
      forAllSystems =
        f:
        nixpkgs.lib.genAttrs nixpkgs.lib.systems.flakeExposed (system: f nixpkgs.legacyPackages.${system});
    in
    {

      packages = forAllSystems (pkgs: {
        default = pkgs.pkgsStatic.rustPlatform.buildRustPackage {
          pname = "zilch";
          version = "1.0.0";

          src = ./.;
          cargoLock.lockFile = ./Cargo.lock;

          nativeBuildInputs = with pkgs.pkgsStatic; [
            wayland-protocols
            wayland
            libxkbcommon
            libGL
            dbus

          ];
        };
      });
      devShells = forAllSystems (pkgs: {

        default = pkgs.mkShell {
          buildInputs = with pkgs; [
            stdenv.cc.cc.lib
            rust-analyzer
            cargo
            rustc
          ];
          LD_LIBRARY_PATH =
            with pkgs;
            lib.makeLibraryPath [
              wayland-protocols
              wayland
              libxkbcommon
              libGL
              dbus
            ];
        };

      });
    };
}
