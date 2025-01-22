---
title: Templates
---

<div class="alert alert-warning" role="alert"><strong>Disclaimer</strong>: Cot is currently missing a lot of features and is <strong>not ready</strong> for production use. This guide is a work in progress and will be updated as Cot matures. That said, you are more than welcome to try it out and provide feedback!</div>

While Cot doesn't enforce use of any specific templating engine, it does provide an integration with an excellent engine called [Rinja](https://rinja.readthedocs.io/). Rinja is a templating engine that is very similar to Jinja2, which is in turn based on Django's templating engine. It is a very powerful engine that allows you to create complex templates with ease, yet providing you with type safety which prevents a lot of errors at compile time.

## Basic syntax

Rinja template is a text file that contains a mix of static text and dynamic content. The dynamic content takes the form of variables, tags, and filters. Here is a simple example of a Rinja template:

```html.j2
<ul>
    {% for item in items %}
    <li>{{ item.title|capitalize }}</li>
    {% endfor %}
</ul>
```

## Read more

This is just a very basic introduction to Rinja. For more information, please refer to the [Rinja documentation](https://rinja.readthedocs.io/).

**This guide chapter is work in progress.**
