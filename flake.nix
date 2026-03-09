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
