# Markdown In Rust

There are many crates for generating markdown in Rust.  I choose the [`comrak`](https://github.com/kivikakk/comrak) crate due to its options and flexibility, and I found it to be fairly fast which doesn't hurt.  It has some great markdown extensions, and is very easy to use (specifying the options is the hardest part, which is not very hard, especially if you use `ComrakOptions::default()`).

### Setup
I have been using an older version of `comrak`, the 0.2.5 version. I believe it is up to 0.2.8 or higher.  If you wish to use the most updated version of Comrak you should know that the 0.2.8 version adds a `default_info_string` field to the options, which takes a `Option<string>`, all code is written using the 0.2.5 version (and thus does not have the `default_info_string` field).

Cargo.toml
```ini
comrak = "0.2.5"
```

main.rs
```rust
extern crate comrak;

use comrak::{markdown_to_html, ComrakOptions};
```

### Usage & Options
The options (as of 0.2.5).  You can use the `markdown_to_html()` method which takes a `&str` containing markdown and a `&ComrakOptions` structure reference.  The `ComrakOptions::default()` can be used to use default values for the options.


The options look like:

```rust
pub const COMRAK_OPTIONS: ComrakOptions = ComrakOptions {
    hardbreaks: true,            // \n => <br>\n
    width: 120usize,             // Column width in characters
    // Use the GitHub style <pre lang="blah"> for fenced code blocks
    github_pre_lang: false,
    
    // If you are using version 0.2.8 uncomment the following:
    // default_info_string: Some(String::from("rust"))
    
    ext_strikethrough: true,     // hello ~world~ person.
    ext_tagfilter: true,         // filters out certain html tags
    ext_table: true,             // | a | b |\n|---|---|\n| c | d |
    ext_autolink: true,          // automatically recognize links
    ext_tasklist: true,          // * [x] Done\n* [ ] Not Done
    ext_superscript: true,       // e = mc^2^
    ext_header_ids: None,        // None / Some("some-id-prefix-".to_string())
    ext_footnotes: true,         // Hi[^x]\n\n[^x]: A footnote here\n
};
````

And that's pretty much it.
- [Comrak GitHub Repository](https://github.com/kivikakk/comrak)
- Version 0.2.5 Docs
    - [Comrak Documentation v0.2.5](https://docs.rs/comrak/0.2.5/comrak/index.html)
    - [ComrakOptions v0.2.5](https://docs.rs/comrak/0.2.5/comrak/struct.ComrakOptions.html)
- Version 0.2.8 Docs
    - [Comrak Documentation v.2.8](https://docs.rs/comrak/0.2.8/comrak/index.html)
    - [ComrakOptions v0.2.8](https://docs.rs/comrak/0.2.8/comrak/struct.ComrakOptions.html)


