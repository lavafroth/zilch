{
  description = "devshell for github:lavafroth/cabinette";

  inputs.flake-utils.url = "github:numtide/flake-utils";

  outputs =
    {
      self,
      nixpkgs,
      flake-utils,
    }:
    flake-utils.lib.eachDefaultSystem (
      system:
      let
        pkgs = import nixpkgs {
          system = system;
          config = {
            allowUnfree = true;
          };
        };
      in
      {
        devShells.default = pkgs.mkShell rec {
          libraries = with pkgs; [
            stdenv.cc.cc.lib
            webkitgtk
            gtk3
            cairo
            gdk-pixbuf
            glib
            dbus
            openssl
            librsvg
          ];
          packages = with pkgs; [
            # rust backend
            at-spi2-atk
            atkmm
            cairo
            gdk-pixbuf
            glib
            gobject-introspection
            gobject-introspection.dev
            gtk3
            harfbuzz
            librsvg
            libsoup_3
            pango
            webkitgtk_4_1
            webkitgtk_4_1.dev
            # dev tools for frontend
            tailwindcss
            vscode-langservers-extracted
          ];

          PKG_CONFIG_PATH =
            with pkgs;
            "${glib.dev}/lib/pkgconfig:${libsoup_3.dev}/lib/pkgconfig:${webkitgtk_4_1.dev}/lib/pkgconfig:${at-spi2-atk.dev}/lib/pkgconfig:${gtk3.dev}/lib/pkgconfig:${gdk-pixbuf.dev}/lib/pkgconfig:${cairo.dev}/lib/pkgconfig:${pango.dev}/lib/pkgconfig:${harfbuzz.dev}/lib/pkgconfig";
        };
      }
    );
}
