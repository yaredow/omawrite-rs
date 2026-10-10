use std::path::Path;
use std::process::Command;

fn main() {
    println!("cargo:rerun-if-env-changed=GSETTINGS_SCHEMAS_PATH");
    println!("cargo:rerun-if-env-changed=GSETTINGS_SCHEMA_DIR");

    let mut schemas_path = std::env::var("GSETTINGS_SCHEMAS_PATH").unwrap_or_default();

    if schemas_path.is_empty() {
        if let Ok(output) = Command::new("nix")
            .args([
                "eval",
                "--raw",
                "--impure",
                "--expr",
                "(let pkgs = import <nixpkgs> {}; in \"${pkgs.gsettings-desktop-schemas}/share/gsettings-schemas/${pkgs.gsettings-desktop-schemas.name}:${pkgs.gtk4}/share/gsettings-schemas/${pkgs.gtk4.name}\")",
            ])
            .output()
        {
            if output.status.success() {
                schemas_path = String::from_utf8_lossy(&output.stdout).trim().to_string();
            }
        }
    }

    if schemas_path.is_empty() {
        let mut dirs = Vec::new();
        if let Ok(entries) = std::fs::read_dir("/nix/store") {
            for entry in entries.flatten() {
                let name = entry.file_name().to_string_lossy().to_string();
                if (name.contains("gsettings-desktop-schemas") || name.contains("-gtk4-"))
                    && !name.ends_with(".drv")
                {
                    let candidate = entry.path().join("share/gsettings-schemas").join(&name);
                    if candidate.join("glib-2.0/schemas").exists() {
                        dirs.push(candidate.to_string_lossy().to_string());
                    }
                }
            }
        }
        schemas_path = dirs.join(":");
    }

    if !schemas_path.is_empty() {
        println!("cargo:rustc-env=NIX_GSETTINGS_PATH={}", schemas_path);

        let schema_dirs: Vec<String> = schemas_path
            .split(':')
            .filter(|p| !p.is_empty())
            .map(|p| format!("{}/glib-2.0/schemas", p))
            .filter(|p| Path::new(p).exists())
            .collect();

        if !schema_dirs.is_empty() {
            println!(
                "cargo:rustc-env=NIX_GSETTINGS_SCHEMA_DIR={}",
                schema_dirs.join(":")
            );
        }
    }

    if let Ok(output) = Command::new("nix")
        .args([
            "eval",
            "--raw",
            "--impure",
            "--expr",
            "(let pkgs = import <nixpkgs> {}; in \"${pkgs.dconf.lib}/lib/gio/modules\")",
        ])
        .output()
    {
        if output.status.success() {
            let gio_mod = String::from_utf8_lossy(&output.stdout).trim().to_string();
            if Path::new(&gio_mod).exists() {
                println!("cargo:rustc-env=NIX_GIO_MODULES={}", gio_mod);
            }
        }
    }
}
