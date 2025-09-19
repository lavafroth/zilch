{
  description = "devshell for github:lavafroth/zilch";

  inputs.flake-utils.url = "github:numtide/flake-utils";

  outputs =
    {
      nixpkgs,
      flake-utils,
      ...
    }:
    flake-utils.lib.eachDefaultSystem (
      system:
      let
        pkgs = import nixpkgs {
          system = system;
        };
      in
      {
        devShells.default = pkgs.mkShell rec {
          libraries = with pkgs; [
            stdenv.cc.cc.lib
            glib
            webkitgtk_4_1
            webkitgtk_4_1.dev
            gobject-introspection
            dbus
            openssl.dev
            librsvg
            libsoup_3
          ];
          packages = with pkgs; [
            # rust backend
            at-spi2-atk
            atkmm
            cairo
            gdk-pixbuf
            glib
            gtk3
            harfbuzz
            pango
            pkg-config
            cargo-tauri
            # dev tools for frontend
            tailwindcss
            vscode-langservers-extracted
          ];

          PKG_CONFIG_PATH =
            with pkgs;
            "${glib.dev}/lib/pkgconfig:${libsoup_3.dev}/lib/pkgconfig:${webkitgtk_4_1.dev}/lib/pkgconfig:${at-spi2-atk.dev}/lib/pkgconfig:${gtk3.dev}/lib/pkgconfig:${gdk-pixbuf.dev}/lib/pkgconfig:${cairo.dev}/lib/pkgconfig:${pango.dev}/lib/pkgconfig:${harfbuzz.dev}/lib/pkgconfig:${openssl.dev}/lib/pkgconfig";
          LD_LIBRARY_PATH = "${pkgs.lib.makeLibraryPath libraries}";
          # https://github.com/tauri-apps/tauri/issues/7354
          XDG_DATA_DIRS = "${pkgs.gsettings-desktop-schemas}/share/gsettings-schemas/${pkgs.gsettings-desktop-schemas.name}:${pkgs.gtk3}/share/gsettings-schemas/${pkgs.gtk3.name}:$XDG_DATA_DIRS";
        };
      }
    );
}
