const CSS_PATH: &'static str = "static/css";

fn main() {
    println!("cargo::rerun-if-changed=scss/main.scss");
    println!("cargo::rerun-if-changed=scss/custom.scss");
    println!("cargo::rerun-if-changed=scss/syntax-highlighting.scss");

    let mut css = grass::from_path("scss/main.scss", &grass::Options::default()).expect("failed to compile SCSS");

    let build_profile = std::env::var("PROFILE").expect("Cargo should set PROFILE");
    if build_profile == "release" {
        let result = {
            let mut stylesheet = lightningcss::stylesheet::StyleSheet::parse(&css, lightningcss::stylesheet::ParserOptions::default()).expect("failed to parse CSS");
            stylesheet.minify(lightningcss::stylesheet::MinifyOptions::default()).expect("failed to minify CSS");
            let mut printer_options = lightningcss::stylesheet::PrinterOptions::default();
            printer_options.minify = true;
            stylesheet.to_css(printer_options).expect("failed to print minified CSS")
        };

        css = result.code;
    }

    std::fs::create_dir_all(CSS_PATH).expect("failed to create CSS directory");
    std::fs::write(format!("{CSS_PATH}/main.css"), css).expect("failed to write CSS");
}
