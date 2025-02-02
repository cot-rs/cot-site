use syntect::parsing::SyntaxSetBuilder;

fn main() {
    build_syntax_highlighting_defs();

    println!("cargo:rerun-if-changed=build.rs");
}

fn build_syntax_highlighting_defs() {
    let mut builder = SyntaxSetBuilder::new();
    builder.add_plain_text_syntax();
    // Syntect parser doesn't work with some newer Sublime Text syntax definitions
    // but newer definitions are more up-to-date (Rust one gains `async` as a
    // keyword, for instance) so we update what we can
    add_syntax_highlighting_from_folder(&mut builder, "Packages/HTML");
    add_syntax_highlighting_from_folder(&mut builder, "Packages/ShellScript");
    add_syntax_highlighting_from_folder(&mut builder, "Packages-new/Rust");
    add_syntax_highlighting_from_folder(&mut builder, "Packages-new/Diff");
    // Third-party syntax definitions
    add_syntax_highlighting_from_folder(&mut builder, "sublime-jinja2");
    add_syntax_highlighting_from_folder(&mut builder, "sublime_toml_highlighting");

    let syntax_set = builder.build();
    syntect::dumps::dump_to_uncompressed_file(&syntax_set, "../syntax-highlighting/defs.bin")
        .expect("failed to dump syntax highlighting defs");
}

fn add_syntax_highlighting_from_folder(builder: &mut SyntaxSetBuilder, path: &str) {
    builder
        .add_from_folder(format!("../syntax-highlighting/{path}"), true)
        .expect(&format!("failed to add {path} syntax highlighting"))
}
