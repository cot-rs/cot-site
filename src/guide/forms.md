---
title: Forms
---

<div class="alert alert-warning" role="alert"><strong>Disclaimer</strong>: Cot is currently missing a lot of features and is <strong>not ready</strong> for production use. This guide is a work in progress and will be updated as Cot matures. That said, you are more than welcome to try it out and provide feedback!</div>

Cot has form processing capabilities that allows you to create forms and handle form submissions with ease. Processing forms is as easy as creating a Rust structure, deriving a trait, and then using one function to process the form using the request data. Cot will automatically validate the form data and handle any errors that occur.

## Form trait

The core of the form processing lies in the `Form` trait inside the `cot::form` module. This trait is used to define the form and the fields that are part of the form. Below is an example of how you can define a form:

```rust
use cot::form::Form;

#[derive(Form)]
struct MyForm {
    name: String,
    email: String,
}
```

And here is how you can process the form inside a request handler:

```rust
use cot::request::Request;
use cot::response::Response;
use cot::status::StatusCode;

async fn my_handler(request: &Request) -> cot::Result<Response> {
    let form = MyForm::from_request(request).await?;

    Ok(Response::new_html(StatusCode::OK, Body::fixed(form.render()?)))
}
```

Before this is really usable, we need to define the form in the HTML template. Thankfully, Cot provides you with a way to implement this easily, too—it can automatically generate the HTML for the form based on the form definition.

**This guide chapter is work in progress.**
