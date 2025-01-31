use syntect::parsing::SyntaxSetBuilder;

const CSS_PATH: &'static str = "static/css";

fn main() {
    build_css();
    build_syntax_highlighting_defs();

    println!("cargo:rerun-if-changed=build.rs");
}

fn build_css() {
    println!("cargo::rerun-if-changed=scss/main.scss");
    println!("cargo::rerun-if-changed=scss/custom.scss");
    println!("cargo::rerun-if-changed=scss/syntax-highlighting.scss");

    let mut css = grass::from_path("scss/main.scss", &grass::Options::default())
        .expect("failed to compile SCSS");

    let build_profile = std::env::var("PROFILE").expect("Cargo should set PROFILE");
    if build_profile == "release" {
        let result = {
            let mut stylesheet = lightningcss::stylesheet::StyleSheet::parse(
                &css,
                lightningcss::stylesheet::ParserOptions::default(),
            )
            .expect("failed to parse CSS");
            stylesheet
                .minify(lightningcss::stylesheet::MinifyOptions::default())
                .expect("failed to minify CSS");
            let mut printer_options = lightningcss::stylesheet::PrinterOptions::default();
            printer_options.minify = true;
            stylesheet
                .to_css(printer_options)
                .expect("failed to print minified CSS")
        };

        css = result.code;
    }

    std::fs::create_dir_all(CSS_PATH).expect("failed to create CSS directory");
    std::fs::write(format!("{CSS_PATH}/main.css"), css).expect("failed to write CSS");
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
    syntect::dumps::dump_to_uncompressed_file(&syntax_set, "syntax-highlighting/defs.bin")
        .expect("failed to dump syntax highlighting defs");
}

fn add_syntax_highlighting_from_folder(builder: &mut SyntaxSetBuilder, path: &str) {
    builder
        .add_from_folder(format!("syntax-highlighting/{path}"), true)
        .expect(&format!("failed to add {path} syntax highlighting"))
}
