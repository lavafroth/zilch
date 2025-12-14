{
  description = "flake for github:lavafroth/shush";

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
          ];
          LD_LIBRARY_PATH = with pkgs; lib.makeLibraryPath [
            wayland-protocols
            wayland
            libxkbcommon
            libGL
          ];
        };

      });
    };
}
