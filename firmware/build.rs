use std::{env, fs, path::PathBuf};

fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=platformio.ini");
    println!("cargo:rerun-if-changed=partitions.csv");
    println!("cargo:rerun-if-changed=sdkconfig.defaults");
    println!("cargo:rerun-if-env-changed=SPOTIFY_TOKEN");
    println!("cargo:rerun-if-env-changed=FISH_AUDIO_API_KEY");
    println!("cargo:rerun-if-env-changed=DC_CORE_HTTP");
    println!("cargo:rerun-if-env-changed=DC_WIFI_SSID");
    println!("cargo:rerun-if-env-changed=DC_WIFI_PASS");
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
    println!("cargo:rerun-if-changed=ui/weather.slint");
    println!("cargo:rerun-if-changed=ui/notes.slint");
    println!("cargo:rerun-if-changed=ui/features.slint");
    println!("cargo:rerun-if-changed=ui/assets/branding/dc-assistant-logo-white.png");
    println!("cargo:rerun-if-changed=ui/assets/icons");

    set_build_env_from_files("DC_CORE_HTTP");
    set_build_env_from_files("DC_WIFI_SSID");
    set_build_env_from_files("DC_WIFI_PASS");
    write_generated_spotify_token();
    write_generated_fish_audio_key();

    slint_build::compile_with_config(
        "ui/main.slint",
        slint_build::CompilerConfiguration::new()
            .embed_resources(slint_build::EmbedResourcesKind::EmbedForSoftwareRenderer),
    )
    .expect("Falha ao compilar arquivos .slint");

    embuild::espidf::sysenv::output();
}

fn set_build_env_from_files(key: &str) {
    if env::var(key)
        .ok()
        .filter(|value| !value.trim().is_empty())
        .is_some()
    {
        return;
    }

    if let Some(value) = read_env_value(".env.local", key).or_else(|| read_env_value(".env", key)) {
        if !value.trim().is_empty() {
            println!("cargo:rustc-env={key}={}", value.trim());
        }
    }
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

fn write_generated_fish_audio_key() {
    let token = env::var("FISH_AUDIO_API_KEY")
        .ok()
        .or_else(|| read_env_value(".env.local", "FISH_AUDIO_API_KEY"))
        .or_else(|| read_env_value(".env", "FISH_AUDIO_API_KEY"))
        .unwrap_or_default();

    let out_dir = PathBuf::from(env::var("OUT_DIR").expect("OUT_DIR nao definido"));
    let contents = format!(
        "pub const FISH_AUDIO_API_KEY: &str = {};\n",
        rust_string_literal(token.trim())
    );
    fs::write(out_dir.join("fish_audio_key.rs"), contents)
        .expect("falha ao gerar fish_audio_key.rs");
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
