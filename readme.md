
# Blogr

Blogr gives you all of the features you need to create your "one blog to rule them all".  Written in [Rust](https://www.rust-lang.org) and powered by the [Rocket framework](https://rocket.rs) to generate pages quickly and efficiently, unlike any PHP based solution (unless you cache everything, thus losing the main benefits).

## Status

Currently the blog works, and for my purposes seems to work fairly well.  However, it needs more testing and code review before it should be used in production.  If you wish to see a demo see [https://vishus.net](https://vishus.net)

## Features

- Stores data in a PostgreSQL database (more databases to be supported in the future, as well as a no database option)
  - This allows full-text searching
- Caches articles from the database
- Tracks unique hits and how many times each user visits each page, as well as totals for each page and a running total for the site
- Compresses the html output, depending on the compression algorithms supported by the client
- Serves static files without needing a reverse proxy setup (although Nginx or Apache in a reverse proxy configuration is recommended for flexibility and customization)
- Supports serving support pages (or content files), that is non-article content that articles can link to.
  - Example: an article needs to link to a code file, or an article needs to link to a photo gallery page.
- Uses Handlebars templates to allow the layout to be customized
- Supports different user roles, default ones are the regular user role and the administrator role.  A user can simultaneously be logged in as multiple roles.
  - Prevents brute force login attempts
  - Passwords are salted and hashed in the database to prevent passwords from being easily retrieved in database dumps
- Has an administration panel that gives administrators access to viewing and editing articles, creating new pages, viewing page statistics, downloading simple and details statistics, downloading database backups, and forcing the database to refresh its caches.
- Uses a configuration file to allow setting up and changing your blog config without needing to go through any code.

More features to come.

## Install

Currently installation requires a working PostgreSQL installation.

See [Setting up a Virtual Private Server](https://vishus.net/content/setup-vps) and [Rocket Configuration](https://vishus.net/content/rust-webserver)

#### PostgreSQL Database Install

- Make sure you have a copy of [the blogr repo](https://github.com/vishusandy/blogr.git) ([view it on githuh](https://github.com/vishusandy/blogr)) and unzip it somewhere
- Once PostgreSQL is installed, import the database provided in the database folder (the default username is admin and password is 'password' - you will want to change this immediately in PostgreSQL).  In a command prompt:
  - CD into the database directory of the git repository downloaded earlier
  - Run the command:
```sql
psql postgresql://postgres@localhost
```
where `postgres@localhost` is the username@address of your install
  - Enter your password
  - Type:
```sql
\i install-database.sql
```
This will install the tables for the blog, enable the pgcrypto extension, and install some stored procedures for password hashing and full-text indexing.

#### Configure Rocket

A quick guide on configuring Rocket can be found at [on my website](https://vishus.net/content/rust-webserver) or [The Rocket Guide](https://rocket.rs/guide/configuration/)

Basically if you are enabling SSL make sure to tell Rocket that in the Rocket.toml file.

#### Configure Blogr

At this point you should have [Rust Installed](https://www.rust-lang.org/en-US/install.html).

Once you have a [Virtual Private Server Installed](https://vishus.net/content/setup-vps) (or some other setup), naviagate to the src folder and edit the settings.rs file.

The first half are the development settings, which is the default mode.  When you pass `--cfg production` to rustc it enables the production mode, which uses the settings in the second half of the settings file.  Most settings should be self explanatory, although better documentation of the configuration settings is currently being worked on.

To configure the app to authenticate to your database, create a file in the src directory called private.rs with the following contents:
```rust
// Development machine user/passwords for database
#[cfg(not(production))]
pub const DATABASE_URL: &'static str = "postgres://username:password@localhost/blog";

// Production machine user/password for database
#[cfg(production)]
pub const DATABASE_URL: &'static str = "postgres://username:password@localhost/blog";

```

The following settings are the ones you must edit:

- The most important settings is the `BLOG_URL` setting which tells the blog the address to use.
- The next 6 settings should be modified to match the `BLOG_URL` setting.
  - **It is not currently recommended to change the route for these settings**, just the make the beginning match the `BLOG_URL` (this needs further work and testing)
- The `INTERNAL_IMGS` should be set to the path of a directory containing images to use in the article headers.
- `DB_BACKUP_SCRIPT` should be set to `scripts\db_backup-dev.bat` for windows and `scripts/db_backup-prod.sh` for linux.

The rest should work as is.

#### Starting Blogr

##### Development
For development mode just run:
```bash
cargo run --release
```
And go to  `http://localhost:8000`  (unless you have configured Rocket.toml differently)

##### Virtual Private Server / Production
For production run:
```bash
rustup run nightly-2018-07-16 cargo rustc --release -- --cfg production
```
- nightly-208-07-16 is the nightly required for Rocket 0.3.15, for other versions see the changelogs in the [Rocket Releases](https://github.com/SergioBenitez/Rocket/releases) or [Rocket 0.3.15 changelog](https://github.com/SergioBenitez/Rocket/blob/v0.3.15/CHANGELOG.md)

Configuring Blogr as a service in Linux will be documented in the future.  This will allow the blog to be started when the server starts up, as well as being able to automatically check the status of the blog and restart it if needed.

## Issues
If there are any problems or questions you can contact me using the contact info at the bottom of [vishus.net](https://vishus.net) or file an issue in the [Blogr GitHub](https://github.com/vishusandy/blogr/issues)