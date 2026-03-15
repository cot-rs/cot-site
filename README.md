<div align="center">
<h1><a href="https://cot.rs">Cot Website Engine</a></h1>

[![Rust Build Status](https://github.com/cot-rs/cot-site/workflows/Rust%20CI/badge.svg)](https://github.com/cot-rs/cot-site/actions/workflows/rust.yml)
[![Docker Build Status](https://github.com/cot-rs/cot-site/workflows/Docker%20Images/badge.svg)](https://github.com/cot-rs/cot-site/actions/workflows/docker.yml)
[![Discord chat](https://img.shields.io/discord/1330137289287925781?logo=Discord&logoColor=white)](https://discord.cot.rs)
</div>

This is the repository for the source of the website for the [Cot web framework](https://github.com/cot-rs/cot).

Note that this repository does not contain the guide pages themselves, and it serves mostly as an engine to build the website and render the Markdown files that are located in the [cot repository](https://github.com/cot-rs/cot/tree/master/docs).

## Submodules
The cot-site repo imports syntax highlighting packages as submodules, which should be pulled initially using the command below before running:

```bash
git submodule update --init --recursive
```

## License

Cot Website is licensed under either of the following, at your option:

* Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or https://www.apache.org/licenses/LICENSE-2.0)
* MIT License ([LICENSE-MIT](LICENSE-MIT) or https://opensource.org/licenses/MIT)

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in Cot Website by you shall be
dual licensed under the MIT License and Apache License, Version 2.0, without any additional terms or conditions.
