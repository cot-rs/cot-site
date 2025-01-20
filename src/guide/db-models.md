---
title: Database models
---

<div class="alert alert-warning" role="alert"><strong>Disclaimer</strong>: Cot is currently missing a lot of features and is <strong>not ready</strong> for production use. This guide is a work in progress and will be updated as Cot matures. that said, you are more than welcome to try it out and provide feedback!</div>

Cot comes with its own ORM (Object-Relational Mapping) system, which is a layer of abstraction that allows you to interact with your database using objects instead of raw SQL queries. This makes it easier to work with your database and allows you to write more maintainable code. It abstracts over the specific database engine that you are using, so you can switch between different databases without changing your code. The Cot ORM is also capable of automatically creating migrations for you, so you can easily update your database schema as your application evolves, just by modifying the corresponding Rust structures.

## Defining models

To define a model in Cot, you need to create a new Rust structure that implements the `Model` trait. This trait requires you to define the name of the table that the model corresponds to, as well as the fields that the table should have. Here's an example of a simple model that represents a link in a link shortener service:

```rust
use cot::db::{model, Auto, LimitedString};

#[model]
pub struct Link {
    id: Auto<i64>,
    #[model(unique)]
    slug: LimitedString<32>,
    url: String,
}
```

There's some very useful stuff going on here, so let's break it down:

* The `#[model]` attribute is used to mark the structure as a model. This is required for the Cot ORM to recognize it as such.
* The `id` field is a typical database primary key, which means that it uniquely identifies each row in the table. It's of type `i64`, which is a 64-bit signed integer. `Auto` wrapper is used to automatically generate a new value for this field when a new row is inserted into the table (`AUTOINCREMENT` or `SERIAL` value in the database nomencalture).
* The `slug` field is marked as `unique`, which means that each value in this field must be unique across all rows in the table. It's of type `LimitedString<32>`, which is a string with a maximum length of `32` characters. This is a custom type provided by Cot that ensures that the string is not longer than the specified length at the time of constructing an instance of the structure.

After putting this structure in your project, you can use it to interact with the database. Before you do that though, it's necessary to create the table in the database that corresponds to this model. Cot CLI has got you covered and can automatically create migrations for you – just run the following command:

```bash
cot make-migrations
```

This will create a new file in your `migrations` directory in the crate's src directory, which looks like so:

**This guide chapter is work in progress.**
