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

          shellHook = ''
            export XDG_DATA_DIRS="$GSETTINGS_SCHEMAS_PATH:$XDG_DATA_DIRS"
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
