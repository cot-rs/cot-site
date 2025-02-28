---
title: FAQ
---

## What websites have been built with Cot?

* [The very website you are looking at right now!](https://cot.rs) ([source code](https://github.com/cot-rs/cot))
* [Mateusz Maćkowski's blog](https://mackow.ski) ([source](https://github.com/m4tx/m4txblog))
* Brewnerator, an experimental beer recipe viewer ([source code](https://github.com/brewnerator/brewnerator))
* An example implementation of server-side detection of using `curl | bash` in the CLI ([source code](https://github.com/m4tx/curl-bash-attack))

Do you have a website built with Cot? Let us know by [opening a pull request](https://github.com/cot-rs/cot-site/edit/master/src/md-pages/faq.md)! We'd love to feature it here.

## Where does the name come from?

There are a few meanings:

* "Cot" is pronounced similarly to a Polish word "kot", which means "cat". Cats are known for their agility and flexibility, and also this is where the logo comes from.
* It is a play on the word "cot" itself, which is a simple bed that is typically used in camping or other outdoor activities. The idea behind the name is that Cot is a web framework that is easy to use and doesn't get in your way.
* You can think of it as an abbreviation for "Comprehensive Online Toolkit".

## What is the difference between Cot and other Rust web frameworks?

The main difference between Cot and most other Rust web frameworks or libraries (such as Axum, Actix, Rocket, Warp, etc.) is that Cot is designed to be a batteries-included web framework that tries to provide the best developer experience possible. This means that Cot provides a wide range of features out of the box, including:

* admin interface
* authentication system
* ORM
* form handling system
* testing framework
* CLI tool
* static file serving system
* templating engine

This makes Cot a great choice for developers who want to build a web application quickly without having to worry about integrating multiple libraries together. This also means that Cot is more opinionated than other frameworks, but it still tries to provide enough flexibility to allow developers to customize their applications as needed.

## What is the difference between Cot and Loco web framework?

Loco is a web framework similar to Cot in a sense that it also tries to provide a batteries-included experience for developers. Cot tries to provide a more developer-friendly experience by providing friendlier APIs and features such as admin panel, or form handling with easy HTML generation. It tries to utilize Rust's type system as much as possible to avoid bugs earlier in the process, hence the choice of Rinja for server-side templates instead of Tera, for instance. Finally, Cot goes as far as creating its own ORM to provide even more tightly integrated experience.

## Why build your own ORM? Isn't this a huge undertaking?

There is a variety ORMs available in Rust, but none of them have the following features:

* Fully automatic migration generation that is database engine agnostic
* Easy to use and readable API that is as close to native Rust as possible
* Admin panel integration

While building an ORM is a huge task, we're in luck, because we are using [sea-query](https://github.com/SeaQL/sea-query), a project that powers the great [SeaORM](https://www.sea-ql.org/SeaORM/) library. This means that we can focus on the features that are important to us, while the heavy lifting is done by the battle tested sea-query library.

## Should I use Cot in production?

While we'd like to say yes, we can't really recommend to run Cot in production. Cot is still in its early stages of development and might be missing features you might want to depend on. API breakages are also expected as we continue to improve the framework. We'd love you to try it out and provide feedback, though!

## Where can I report a bug or suggest an enhancement in Cot?

You can report bugs or suggest enhancements by opening an issue on the [Cot GitHub repository](https://github.com/cot-rs/cot/issues). We'd love to hear your feedback!

## Where can I report an issue in the Cot website?

Please use [cot-site GitHub repository issue tracker](https://github.com/cot-rs/cot-site/issues) to report any issues with the website. You can also open a pull request if you'd like to contribute to the website!

## Was Cot covered in any media?

Yes! Here are some articles and videos about Cot:

* [Cot framework aims to ease Rust web development](https://www.infoworld.com/article/3832992/cot-framework-aims-to-ease-rust-web-development.html) on [InfoWorld](https://www.infoworld.com/)
* [Welcome, Cot: the Rust web framework for lazy developers](https://mackow.ski/blog/cot-the-rust-web-framework-for-lazy-developers/) on [Mateusz Maćkowski's blog](https://mackow.ski/) _(written by us)_ 
