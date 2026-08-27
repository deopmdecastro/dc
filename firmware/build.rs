// build.rs — compila os módulos Slint e integra o toolchain esp-idf.
fn main() {
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
