---
title: Introduction
---

[bacon]: https://dystroy.org/bacon/

Cot is a free and open-source web framework for Rust that's been designed to enable rapid development, fast iteration, as well as secure and bug-free web applications. It is inspired by [Django](https://www.djangoproject.com/), and aims to provide a similar experience to developers who are already familiar with it, but also to be easy to learn for those who are not.

## Who is this guide for?

This guide doesn't assume any advanced knowledge in Rust or web development in general (although this will help, too!). It's aimed at beginners who are looking to get started with Cot, and will guide you through the process of setting up a new project, creating your first views, using the Cot ORM and running your application.

If you are not familiar with Rust, you might want to start by reading the [Rust Book](https://doc.rust-lang.org/book/), which is an excellent resource for learning Rust. 

## Installing and running Cot CLI

The easiest way to get started with Cot is to use the Cot CLI. First, make sure you have `cargo` installed – if not, use [rustup](https://rustup.rs/). Then, you can install Cot CLI by running the following command:

```bash
cargo install --locked cot-cli
```

Then you can create your first Cot project:

```bash
cot new cot_tutorial
```

This creates a new directory called `cot_tutorial` with a new Cot project inside. Let's see what's inside:

```
cot_tutorial
├── src
│   └── main.rs
├── static
│   └── css
│       └── main.css
├── templates
│   └── index.html
├── .gitignore
├── bacon.toml
└── Cargo.toml
```

Apart from the typical Rust project structure, you can see that Cot has created a `static` directory for your static files (like CSS, JavaScript, images, etc.), a `templates` directory for your HTML templates, and a `bacon.toml` file, which is a configuration for [bacon], a tool that handles reloading your application when the source code changes. 

If you don't have [bacon] installed already, we strongly recommend you to do so. It will make your development process much more pleasant. You can install it by running:

```bash
cargo install --locked bacon
```

After you do that, you can run your Cot application by running:

```bash
bacon serve
```

Or, if you don't have bacon installed, you can run your application with the typical:

```bash
cargo run
```

Now, if you open your browser and navigate to [`localhost:8080`](http://localhost:8080), you should see a welcome page that Cot has generated for you. Congratulations, you've just created your first Cot application!

## Views

**This guide chapter is work in progress.**
