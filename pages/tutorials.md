
# Featured Content


##### Rust Code
- [accept.rs]({{base_url}}content/accept.rs) -Determines what compression methods the client accepts
- [xpress.rs]({{base_url}}content/xpress.rs) - Allows modifying response headers, client side caching, and adding compression.
- [login.rs]({{base_url}}content/login.rs) - An example implementation of using Rocket-auth-login crate to allow an administrator user.
- [Rust Snippets](rust-snippets) - A collection of helpful Rust code snippets

##### About This Website
- [Website Details]({{base_url}}content/about-site) - Libraries and services that power this website
- [Blog Organization]({{base_url}}content/blog-organization) - An overview of how the blog works and some of the more interesting modules.
- [Blog GitHub Repo](https://github.com/vishusandy/blogr) - The GitHub repository.  Code licensed under the MIT license.  The theme/layout is &copy; 2018 Andrew Prindle, contact me about using it (see email link at bottom of page).
- [Rust-Auth-Login Repo](https://github.com/vishusandy/rocket-auth-login) - This crate can be used to help with login and authentication.

# All Tutorials & Code
## Rust Web Apps
- [Setup VPS]({{base_url}}content/setup-vps) - Setup a Virtual Private Server
- [VPS Rust Setup]({{base_url}}content/rust-webserver) - Install Rust and configure Rocket web apps
- [Rocket Web Apps]({{base_url}}content/rust-rocket-web-apps) - Tutorial on web app design and example modules to help you get started with your own Rocket powered web app written in Rust
- [Rust-Auth-Login]({{base_url}}content/rocket-auth-login) - Explains how to use the Rocket-auth-login module to create web apps with authentication and multiple user roles
- [Rust-Auth-Login Repo](https://github.com/vishusandy/rocket-auth-login) - The GitHub repository for the RocketAuthLogin code.  This crate can be used to help with login and authentication.
- 

## Rust Code
The following source code files may provide useful inspiration for your own projects.

- [accept.rs]({{base_url}}content/accept.rs) - A module to determine what compression methods the client accepts as well as find the preferred compression method to use.
- [xpress.rs]({{base_url}}content/xpress.rs) - A module providing easy manipulation of response headers (example: adding expiration headers to allow files to be cahced) and compressing output based on the client's accepted compression methods (see accept.rs above).
- [collate.rs]({{base_url}}content/collate.rs) - A configurable module for pagination.  It allows you to specify how many pages you want shown from the beginning and end as well as how many pages to show before and after the current page.
- [counter.rs]({{base_url}}content/counter.rs) - A module demonstrating a possible approach to tracking page hits.
- [database.rs]({{base_url}}content/database.rs) - A module to allow access to an R2D2 connection pool from within route functions.
- [rocket-dbconn-example.rs]({{base_url}}content/rocket-dbconn-example.rs) - An example of using the database.rs module above
- [login.rs]({{base_url}}content/login.rs) - An example implementation of using Rocket-auth-login crate to allow an administrator user.
- [Cargo.toml]({{base_url}}content/Cargo.toml) - An example Cargo.toml file with many wonderful crates that you should check out.

## Rust Miscellaneous
- [Rust Snippets](rust-snippets) - A collection of helpful Rust code snippets


## Linux And Webservers
- [Setup VPS]({{base_url}}content/setup-vps) - Setup a webserver on a Virtual Private Server
- [Linux Tips]({{base_url}}content/linux-tips) - Some helpful commands for managing a linux webserver

# About Me
- [Sublime Theme]({{base_url}}content/sublime-theme) - The custom Sublime Text theme I use for Rust programming
- [About Me]({{base_url}}content/about-me) - A list of programs and tools I use as well as some information about me
- [Website Details]({{base_url}}content/about-site) - A list of libraries and services that were used to create this website
- [Blog Organization]({{base_url}}content/blog-organization) - An overview of how the blog works and some of the more interesting modules.  The modules described here may be useful in inspiring code for other projects.
- [Blog GitHub Repo](https://github.com/vishusandy/blogr) - The GitHub repository.  Code licensed under the MIT license.  The theme/layout is &copy; 2018 Andrew Prindle, contact me about using it (see email link at bottom of page).
