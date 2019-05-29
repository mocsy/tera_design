
# A webapp to use tera templates for frontend development
Tera design was envisioned with frontend developers in mind.
It is a dev web server focusing on rapid frontend protoyping using the [tera](https://crates.io/crates/tera) templates engine.
Writen in rust using [actix-web](https://crates.io/crates/actix-web/1.0.0-rc), it is so fast you never gonna notice any delays.

# How it works
Tera design looks for .html files in the templates directory.
When you open a page in the browser you can ommit the .html extension.
If a template requires variables as input, then those can be put into a .ctx file of the same name in JSON format.
In case a required value is not present in the corresponding .ctx file, it shows an error in the browser.
This allows using tera to enhance static html design efforts and makes it easy to integrate the result to tera based rust projects.

# Example
It includes a copy of (SB Admin 2)(https://github.com/BlackrockDigital/startbootstrap-sb-admin-2) modified to showcase some tera template fetures without the need for completeness.

## License
Code released under the MIT license.