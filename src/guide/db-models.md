---
title: Database models
---

<div class="alert alert-warning" role="alert"><strong>Disclaimer</strong>: Cot is currently missing a lot of features and is <strong>not ready</strong> for production use. This guide is a work in progress and will be updated as Cot matures. That said, you are more than welcome to try it out and provide feedback!</div>

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
* The `id` field is a typical database primary key, which means that it uniquely identifies each row in the table. It's of type `i64`, which is a 64-bit signed integer. `Auto` wrapper is used to automatically generate a new value for this field when a new row is inserted into the table (`AUTOINCREMENT` or `SERIAL` value in the database nomenclature).
* The `slug` field is marked as `unique`, which means that each value in this field must be unique across all rows in the table. It's of type `LimitedString<32>`, which is a string with a maximum length of `32` characters. This is a custom type provided by Cot that ensures that the string is not longer than the specified length at the time of constructing an instance of the structure.

After putting this structure in your project, you can use it to interact with the database. Before you do that though, it's necessary to create the table in the database that corresponds to this model. Cot CLI has got you covered and can automatically create migrations for you – just run the following command:

```bash
cot make-migrations
```

This will create a new file in your `migrations` directory in the crate's src directory. We will come back to the contents of this file later in this guide, but for now, let's focus on how to use the model to interact with the database.

## Common operations

### Saving models

In order to write a model instance to the database, you can use the `save` method. Note that you need to have an instance of the `Database` structure to do this – typically you can get it from the request object in your view. Here's an example of how you can save a new link to the database inside a view:

```rust
async fn create_link(request: Request) -> cot::Result<Response> {
    let mut link = Link {
        id: Auto::default(),
        slug: LimitedString::new("slug").unwrap(),
        url: "https://example.com".to_string(),
    };
    link.save(request.db()).await?;

    // ...
}
```

### Updating models

Updating a model is similar to saving a new one, but you need to have an existing instance of the model that you want to update, or another instance with the same primary key. Here's an example of how you can update an existing link in the database:

```rust
link.url = "https://example.org".to_string();
link.save(request.db()).await?;
```

Note that `.save()` is a convenient method that can be used for both creating new rows and updating existing ones. If the primary key of the model is set to `Auto`, the method will always create a new row in the database. If the primary key is set to a specific value, the method will update the row with that primary key, or create a new one if it doesn't exist.

If you specifically want to update a row in the database for given primary key, you can use the `update` method:

```rust
link.url = "https://example.org".to_string();
link.update(request.db()).await?;
```

Similarly, if you want to insert a new row in the database and cause an error if a row with the same primary key already exists, you can use the `insert` method:

```rust
let mut link = Link {
    id: Auto::default(),
    slug: LimitedString::new("slug").unwrap(),
    url: "https://example.com".to_string(),
};
link.insert(request.db()).await?;
```

### Retrieving models

### Deleting models

## Foreign keys

## Model attributes

## Migration files

```rust
//! Generated by cot CLI 0.1.0 on 2025-01-23 11:00:12+00:00

#[derive(Debug, Copy, Clone)]
pub(super) struct Migration;
impl ::cot::db::migrations::Migration for Migration {
    const APP_NAME: &'static str = "cot-test";
    const MIGRATION_NAME: &'static str = "m_0001_initial";
    const DEPENDENCIES: &'static [::cot::db::migrations::MigrationDependency] = &[];
    const OPERATIONS: &'static [::cot::db::migrations::Operation] = &[
        ::cot::db::migrations::Operation::create_model()
            .table_name(::cot::db::Identifier::new("link"))
            .fields(
                &[
                    ::cot::db::migrations::Field::new(
                            ::cot::db::Identifier::new("id"),
                            <cot::db::Auto<i64> as ::cot::db::DatabaseField>::TYPE,
                        )
                        .auto()
                        .primary_key()
                        .set_null(
                            <cot::db::Auto<i64> as ::cot::db::DatabaseField>::NULLABLE,
                        ),
                    ::cot::db::migrations::Field::new(
                            ::cot::db::Identifier::new("slug"),
                            <cot::db::LimitedString<
                                32,
                            > as ::cot::db::DatabaseField>::TYPE,
                        )
                        .set_null(
                            <cot::db::LimitedString<
                                32,
                            > as ::cot::db::DatabaseField>::NULLABLE,
                        )
                        .unique(),
                    ::cot::db::migrations::Field::new(
                            ::cot::db::Identifier::new("url"),
                            <String as ::cot::db::DatabaseField>::TYPE,
                        )
                        .set_null(<String as ::cot::db::DatabaseField>::NULLABLE),
                ],
            )
            .build(),
    ];
}

#[derive(::core::fmt::Debug)]
#[::cot::db::model(model_type = "migration")]
struct _Link {
    id: cot::db::Auto<i64>,
    #[model(unique)]
    slug: cot::db::LimitedString<32>,
    url: String,
}
```

**This guide chapter is work in progress.**
