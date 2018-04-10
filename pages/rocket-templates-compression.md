
# Rocket Templates And Compression

I highly recommend using Rocket's templates, however they are not very flexible and have some drawbacks.  Due to its design you cannot easily retrieve the rendered contents of a template quickly.  The `to_string()` method is very slow, and is not recommended to be used for anything other than running tests.

### The Problem
So you have a route that returns a `Template`.  Awesome!  But what if you want to make some changes to the HTTP response?  Maybe you want to have the client cache it for 7 days, or maybe you want to compress the contents of the template?

You could use fairings, and maybe you should depending on your use case, but those run on every request regardless of whether they need to.

### The Solution

- [The xpress module]({{base_url}}content/xpress.rs)

I created the [xpress module]({{baes_url}}content/xpress.rs) for my blog app for exactly this.  I have not yet made it into a separate crate as I am not convinced it is ready for that, but it works well for my purposes.  You are free to use it in your Rocket application or just use it for inspiration for your own solution.

The module allows you to have routes that generate some content in the form of a template, String, byte vector, or even file; then add compression, client side caching, or even make the contents an attachment.


### Using Xpress

The `xpresss` module also requires the `accept` module to determine which compression methods the client supports
- [Accept Module]({{base_url}}content/accept.rs)

The brotli and libflate crates are also used to handle compression.
Cargo.toml
```ini
brotli = "1.0.9"
libflate = "0.1.12"
```

There are a few crates to import/use besides the basic Rocket imports/use statements:

```rust
extern crate libflate;
extern crate brotli;

use rocket_contrib::Template;
use rocket::response::content::Html;
use rocket::response::{NamedFile, Redirect};

// You may optionally import hyper headers if you need
// to modify the http headers in a more advanced way
use rocket::http::hyper::header::{Headers, ContentDisposition, DispositionType, DispositionParam, Charset};

```

I tried few other comrpession crates and found these to be the fastest in the benchmarks I performed (the zopfli crate was particularly slow for me).  Using the compression settings specified in the `xpress` module gzip compression can compress 25kb in just a few miliseconds.  Brotli seems to give better compression results at the cost of an extra milisecond or two occassionaly (but it was sometimes faster so go figure).

### How It Works
The module defines a type named `Express` that holds some content (String, bytes, file, or, Template) as well as information for modifying the HTTP headers.

The `Express` structure implements Rocket's `Respodner` trait so your routes can return the Express method instead of an HTML String or a Template.

To convert data into an `Exprses` structure you use the `into()` or `from()` functions.  The `Express` structure implements the `From` trait for Strings, NamedFiles, byte vectors, Templates, and PathBufs (it retrieves the contents of the file for you).  In plain english?
```rust
#[get("/about")]
pub fn about() -> Exprses {
    let body: Template = get_some_content("about");
    let express: Exprses = body.into();
    express
}
```

### Compression
Compressing Templates was one of most compelling reasons for creating this module.  Once you have an `Express` instance you can modify it before returning it like:

```rust
#[get("/about")]
pub fn about(encoding: AcceptCompression) -> Exprses {
    let body: Template = get_some_content("about");
    let express: Exprses = body.into();
    express.compress( encoding )
}
```

That's pretty easy isn't it?  You take whatever data your want to return and call `into()` on it (assuming `Exprses` implements the `From` trait for that datatype - and if not you could always implement it yourself, the code is all there) then just return it, while optionally calling methods to modify the HTTP response.

The above example uses the [`accept`]({{base_url}}content/accept.rs) module to determine which compression methods the client supports.  The module is fairly simple, and you may want to customize the `preferred()` function to change the preferred order of compression algorithms to be optimized for your clients.

#### Supported Compression Methods
The `xpress` module uses the brotli and libflate crates as well as the `accept` module.

- [{{base_url}}content/accept.rs]({{base_url}}content/accept.rs)

### Expiration Headers
The `set_ttl()` method modifies an `Express` instance by setting expriation headers to a specified number of seconds, or prevents caching when the value passed to it is -1.  Any value less than -1 does not set any caching related headers at all.

The module, by default, adds expiration headers to `NamedFile`s and `PathBuf`s, the exact amount of time depends on the constant defined at the beginning.  You can always prevent client side caching on files with `exprses.set_ttl(-1)`.

### Downloading Files
You can even use the `xpress` module to have content downloaded by a browser instead of displayed.

```rust
#[get("/download")]
pub fn download() -> Express {
    // Import hyper http headers
    use rocket::http::hyper::header::{Headers, ContentDisposition, DispositionType, DispositionParam, Charset};

    let content: String = get_download();
    // specify the filename the browser should save the file as
    let filename: Vec<u8> = b"Filename.txt";
    
    // Create the header
    let attachment = ContentDisposition {
        disposition: DispositionType::Attachment,
        parameters: vec![DispositionParam::Filename(
            // The character set for the bytes of the filename
            Charset::Iso_8859_1, 
            // The optional language tag (see `language-tag` crate)
            None, 
            // the name to save the file as, in bytes
            filename 
        )]
    };
    
    let express: Express = content.into();
    express
        .set_ttl(-2)
        .add_header(attachment)
    // Use .set_ttl(-2) to completely disable cache headers
    // IE may break when downloading a file over HTTPS with cache-control headers
    
}

```

### Markdown
This is just a little side note, and a personal recommendation: if you are making a blog or a website with a lot of content I would highly recommend using markdown.  I have used the `Comrak` crate and found it to be fast and powerful.
[Using Markdown with Comrak in Rust]({{base_url}}content/markdown-comrak)

<!--
The solution presented here is just my approach to the problem, it may not be suitable for you, but maybe it is.
-->






