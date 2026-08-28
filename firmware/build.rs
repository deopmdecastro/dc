// build.rs — compila os módulos Slint e integra o toolchain esp-idf.
fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=platformio.ini");
    println!("cargo:rerun-if-changed=partitions.csv");
    println!("cargo:rerun-if-changed=sdkconfig.defaults");
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
    println!("cargo:rerun-if-changed=ui/store.slint");
    println!("cargo:rerun-if-changed=ui/assets/branding/dc-assistant-logo-white.png");
    println!("cargo:rerun-if-changed=ui/assets/icons");

    // Compila a árvore Slint (entrypoint: ui/main.slint).
    slint_build::compile_with_config(
        "ui/main.slint",
        slint_build::CompilerConfiguration::new()
            .embed_resources(slint_build::EmbedResourcesKind::EmbedForSoftwareRenderer),
    )
    .expect("Falha ao compilar arquivos .slint");

    // Embed the Spotify OAuth token at build time if provided.
    if let Ok(token) = std::env::var("SPOTIFY_TOKEN") {
        println!("cargo:rustc-env=SPOTIFY_TOKEN={token}");
    }

    // Encadeia a build do esp-idf/PlatformIO.
    embuild::espidf::sysenv::output();
}
