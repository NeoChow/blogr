
<!-- https://vishus.net/content/xpress.rs -->

# Templates in Rocket Routes

If you have used Rocket's templates you know how great they are for performing their task.  They are fast and make it very easy to return HTML webpages.  I would highly recommend using templates if you are using Rocket to generate anything other than very basic content or static files.

You can include other handlebars templates like `{{> some_template}}` and you can now even add handlebars helper functions:
```rust
type HelperResult = Result<(), RenderError>;

fn wow_helper(h: &Helper, _: &Handlebars, rc: &mut RenderContext) -> HelperResult {
    if let Some(param) = h.param(0) {
        write!(rc.writer, "<b><i>{}</i></b>", param.value().render())?;
    }
    Ok(())
}

fn rocket() -> rocket::Rocket {
    rocket::ignite()
        .mount("/", routes![index, get])
        .catch(catchers![not_found])
        .attach(Template::custom(|engines| {
            engines.handlebars.register_helper("wow", Box::new(wow_helper));
        }))
}
```

Taken from [Rocket's Handlebars Example](https://github.com/SergioBenitez/Rocket/blob/master/examples/handlebars_templates/src/main.rs)




