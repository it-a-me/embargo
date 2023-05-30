use slint_build::CompilerConfiguration;

fn main() {
    slint_build::compile_with_config(
        "ui/main.slint",
        CompilerConfiguration::new()
            .embed_resources(slint_build::EmbedResourcesKind::EmbedForSoftwareRenderer),
    )
    .unwrap();
}
