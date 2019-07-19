
# A webapp to use tera templates for frontend development
Tera design was envisioned with frontend developers in mind.
It is a dev web server focusing on rapid frontend protoyping using the [tera](https://crates.io/crates/tera) templates engine.
Writen in rust using [actix-web](https://crates.io/crates/actix-web/1.0.5), it is so fast you never gonna notice any delays.
In fact an artifical delay feature is being considered.

# How it works
Tera design looks for .html files in the templates directory.
When you open a page in the browser you can ommit the .html extension.

If a template requires variables as input, then those can be put into a .json file of the same name in JSON format.
In case a required value is not present in the corresponding .json file, it shows an error in the browser.

Templates can be organized into directories, and these directories make it into the url of the page.
The resulting prototype uses these urls and handled and feels like the final app would do.

When parts of the variables apply to every page in a given directory, these can be put into a single mod.json file instead of copy pasting the same values to every context.

The above features enable tera_design to enhance static html design efforts. It easy to integrate the result to jinja2 based projects, like rust tera/askama, python django, etc.

# Example
It includes a copy of [SB Admin 2](https://github.com/BlackrockDigital/startbootstrap-sb-admin-2) modified to showcase some tera template fetures without the need for completeness.

## License
Code released under the MIT license.

## Build
cargo install cross
cross build --release --target x86_64-pc-windows-gnu
