use std::{env, fs, path::PathBuf};

fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=platformio.ini");
    println!("cargo:rerun-if-changed=partitions.csv");
    println!("cargo:rerun-if-changed=sdkconfig.defaults");
    println!("cargo:rerun-if-env-changed=SPOTIFY_TOKEN");
    println!("cargo:rerun-if-changed=.env");
    println!("cargo:rerun-if-changed=.env.local");
    println!("cargo:rerun-if-changed=ui/main.slint");
    println!("cargo:rerun-if-changed=ui/theme.slint");
    println!("cargo:rerun-if-changed=ui/status_bar.slint");
    println!("cargo:rerun-if-changed=ui/splash.slint");
    println!("cargo:rerun-if-changed=ui/first_boot.slint");
    println!("cargo:rerun-if-changed=ui/chat_view.slint");
    println!("cargo:rerun-if-changed=ui/app_launcher.slint");
    println!("cargo:rerun-if-changed=ui/music_player.slint");
    println!("cargo:rerun-if-changed=ui/settings.slint");
    println!("cargo:rerun-if-changed=ui/control_center.slint");
    println!("cargo:rerun-if-changed=ui/alarm.slint");
    println!("cargo:rerun-if-changed=ui/assets/branding/dc-assistant-logo-white.png");
    println!("cargo:rerun-if-changed=ui/assets/icons");

    write_generated_spotify_token();

    slint_build::compile_with_config(
        "ui/main.slint",
        slint_build::CompilerConfiguration::new()
            .embed_resources(slint_build::EmbedResourcesKind::EmbedForSoftwareRenderer),
    )
    .expect("Falha ao compilar arquivos .slint");

    embuild::espidf::sysenv::output();
}

fn write_generated_spotify_token() {
    let token = env::var("SPOTIFY_TOKEN")
        .ok()
        .or_else(|| read_env_value(".env.local", "SPOTIFY_TOKEN"))
        .or_else(|| read_env_value(".env", "SPOTIFY_TOKEN"))
        .unwrap_or_default();

    let out_dir = PathBuf::from(env::var("OUT_DIR").expect("OUT_DIR nao definido"));
    let contents = format!(
        "pub const SPOTIFY_TOKEN: &str = {};\n",
        rust_string_literal(token.trim())
    );

    fs::write(out_dir.join("spotify_token.rs"), contents).expect("falha ao gerar spotify_token.rs");
}

fn read_env_value(path: &str, key: &str) -> Option<String> {
    let content = fs::read_to_string(path).ok()?;
    for line in content.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }

        let Some((name, value)) = line.split_once('=') else {
            continue;
        };

        if name.trim() == key {
            return Some(unquote(value.trim()).to_owned());
        }
    }

    None
}

fn unquote(value: &str) -> &str {
    value
        .strip_prefix('"')
        .and_then(|v| v.strip_suffix('"'))
        .or_else(|| value.strip_prefix('\'').and_then(|v| v.strip_suffix('\'')))
        .unwrap_or(value)
}

fn rust_string_literal(value: &str) -> String {
    format!("{value:?}")
}
