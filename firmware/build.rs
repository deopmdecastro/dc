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

    // Compila a árvore Slint (entrypoint: ui/main.slint).
    slint_build::compile_with_config(
        "ui/main.slint",
        slint_build::CompilerConfiguration::new()
            .embed_resources(slint_build::EmbedResourcesKind::EmbedForSoftwareRenderer),
    )
    .expect("Falha ao compilar arquivos .slint");

    // Encadeia a build do esp-idf/PlatformIO.
    embuild::espidf::sysenv::output();
}
