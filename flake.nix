{
  description = "omawrite-rs";

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixos-unstable";
  };

  outputs = { self, nixpkgs }:
    let
      systems = [ "x86_64-linux" "aarch64-linux" ];
      forEachSystem = f: nixpkgs.lib.genAttrs systems (system: f (import nixpkgs { inherit system; }));
    in
    {
      devShells = forEachSystem (pkgs: {
        default = pkgs.mkShell {
          nativeBuildInputs = with pkgs; [
            wrapGAppsHook4
            pkg-config
            cargo
            rustc
            rust-analyzer
            rustfmt
            clippy
          ];

          buildInputs = with pkgs; [
            gsettings-desktop-schemas
            glib
            dconf
            gtk4
            libadwaita
            gtksourceview5
            pango
            cairo
            gdk-pixbuf

            libxkbcommon
            wayland
            libGL
          ];

          GSETTINGS_SCHEMAS_PATH = "${pkgs.gsettings-desktop-schemas}/share/gsettings-schemas/${pkgs.gsettings-desktop-schemas.name}:${pkgs.gtk4}/share/gsettings-schemas/${pkgs.gtk4.name}";
          GSETTINGS_SCHEMA_DIR = "${pkgs.gsettings-desktop-schemas}/share/gsettings-schemas/${pkgs.gsettings-desktop-schemas.name}/glib-2.0/schemas:${pkgs.gtk4}/share/gsettings-schemas/${pkgs.gtk4.name}/glib-2.0/schemas";

          shellHook = ''
            export GSETTINGS_SCHEMA_DIR="$GSETTINGS_SCHEMA_DIR"
            export XDG_DATA_DIRS="$GSETTINGS_SCHEMAS_PATH:$XDG_DATA_DIRS"
            export GIO_EXTRA_MODULES="${pkgs.dconf.lib}/lib/gio/modules:$GIO_EXTRA_MODULES"
            export LD_LIBRARY_PATH="${pkgs.lib.makeLibraryPath (with pkgs; [
              libGL
              libxkbcommon
              wayland
            ])}:$LD_LIBRARY_PATH"

            echo "🦀 Rust development environment ready!"
            echo "   cargo: $(cargo --version)"
            echo "   rustc: $(rustc --version)"
          '';
        };
      });
    };
}
