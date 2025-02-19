<div align="center">
<h1><a href="https://cot.rs">Cot Website</a></h1>

[![Rust Build Status](https://github.com/cot-rs/cot-site/workflows/Rust%20CI/badge.svg)](https://github.com/cot-rs/cot-site/actions/workflows/rust.yml)
[![Docker Build Status](https://github.com/cot-rs/cot-site/workflows/Docker%20Images/badge.svg)](https://github.com/cot-rs/cot-site/actions/workflows/docker.yml)
[![Discord chat](https://img.shields.io/discord/1330137289287925781?logo=Discord&logoColor=white)](https://discord.cot.rs)
</div>

This is the repository for the source of the website for the [Cot web framework](https://github.com/cot-rs/cot).

## Development

Make sure you have `cargo` installed. You can get it through [rustup](https://rustup.rs/).

Then, the easiest way to run the development server is to run:

```bash
cargo run
```

The website doesn't need any external resources (such as the database), so nothing more is needed.

### Modifying the guide or other Markdown files

Because of the internals of Markdown processing macros work, you will need to use the nightly toolchain and enable the `nightly` features if you want to see the changes made to the Markdown files in the without using `cargo clean`:

```bash
cargo +nightly run --features nightly
```

### Live reloading

To make the development more convenient, you can use [bacon](https://dystroy.org/bacon/) to get live reloading capabilities. After installing it, you can execute:

```bash
bacon serve
```

All the changes you do in Rust source files or the templates should be automatically reflected in the web browser.

## License

Cot Website is licensed under either of the following, at your option:

* Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or https://www.apache.org/licenses/LICENSE-2.0)
* MIT License ([LICENSE-MIT](LICENSE-MIT) or https://opensource.org/licenses/MIT)

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in Cot Website by you shall be
dual licensed under the MIT License and Apache License, Version 2.0, without any additional terms or conditions.
