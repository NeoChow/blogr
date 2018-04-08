# How The Blog Works
The project makes heavy use of Rocket's request guards and responders as well as Handlebars templates.



### Caching
For performance reasons the blog fetches content from a database at startup and accesses the data on request. The main() function will initialize caches from the following modules:
- cache module
- content module

### Static Pages
The `content` module exists to add non-article pages to the website, pages intended as support pages to hold content not suitable for articles but may be linked to from articles.  For example, if you had an article that needed to show some content, but the content would not fit well in the article, you could put the content in a static page and link to it from the article.

The module holds structures and methods that load and cache files in the `pages` folder.  All markdown files will be processed as markdown.  Markdown documents can have optional metadata at the top of the document separated by at least 5 hyphens on a separte line.

The static pages can be code files, markdown documents, or just plain html/text that you wish to display.  The blog will try to extract metadtaa from files with a .md or .page extension.  By explicitly specifying the template the page can be made to display no layout, a regular layout, or a layout optimized for displaying code files.  The blog will attempt to find the best layout based on the extension if the file does not have a .md or .html extension.

The following metadata is extracted (some items accept multiple names, those items are separated by a forward slash /):
- **uri/url** * - the 
- **title** * - the title of the page
- **template** - the name of the template to use, without the extension (example: .html.hbs) at the end.  This should be either page-blank-template for showing only the content and no layout, page-code-template for code files, or page-template for a regular page layout.
- **js/javascript/script** - some optional javascript code to be executed at the bottom of the page
- **desc/description** - a short description of the page, used for the &lt;meta&gt; tag and may have further use in the future.
- **admin/administrator** - a bool indicating whether the page requires the client to be logged in as an administrator (see `ral_administrator` module) to view the contents
- **user/logged_in** - a bool indicating whether the page requires the client to be logged in as a user (see `ral_users` module) to view the contents
- **menu/menu_basic** - json serialized Vec&lt;TemplateMenu&gt; indicating what basic menu items to display.  The default value uses the DEFAULT_PAGE_MENU setting, which is also used in the `template` module.
- **menu-dropdown/dropdown-menu** - same as the above menu/basic_menu except it displays dropdown menu items.  It uses the DEFAULT_PAGE_DROPDOWN setting as a default value.
- **dropdown/dropdown-name** - the name of the dropdown menu, if it is specified above
- **markdown** - a bool indicating whether to parse the document as metadata.  This defaults to true.

Note: The * indicates a required field.  The title and uri fields are required.  For 

### Hit Counter and Statistics
The main() function also initializes a hit counter and a HashMap of HashMaps to store the number of times each IP address vistied each page.

On start the program will look for files containing serialized hit counts and IP address logs, and if found files are deserialized and used as the values for the hit counters, otherwise the HashMap is empty.`

### Database Connection Pool
An R2D2 postgres database connection pool is initialized at startup.  This connection pool allows for faster and more efficient database access where needed (user logins and admin functionality).

### Page Routes
The `pages` module holds many of the functions used by Rocket to display pages.  The `routes` module provides functions for the newer versions of routes that make use of the cached Articles and text.

The functions in the `routes` module just call functions (usually #[inlined]'d) `serve()` functions from the `cache::pages` module.



### Compression & HTTP Headers
The `xpress` module provides an `Express` structure to convert and display: Strings, Rocket's Templates, byte vectors, and even files.

The `Express` structure is used to return data from a route function which will be displayed as HTML or text, depending on the type of data.  The `Express` structure applies some default HTTP headers to the response, while allowing custom headers to be attached.

The `accept` module allows the blog to determine which compression methods the client supports and chooses the best method.  Then the `xpress` module can apply compression.

### Collate
The `collate` module provides pagination information for pages that require multiple pages.

The `Page` type holds an instance of the `Collate` trait, which is used mainly to determine the default number of items per page and other customizable settings, the `Pagination` struct uses the trait default of 10 items per page.

When a route function specifies a parameter of the type `Page<Pagination>` (or `Page<T: COllate>`) the `collate` module will automatically look for a get query string with the page number and a non-default items per page.

##### Collate in Templates
Once the pagination info is passed to the templating functions in the `templates` module, the `collate` module will generate navigation links and an 'items per page' selection box.  The only extra piece of information required is the total number of items in the collection.

You send the article to be displayed on that page (determined with help from `Page`'s `page_data()` method) along with the `Page<T: Collate>`, total number of items, and an optional message to display (the default is just calling `Page`'s `page_info()` method) to the templating function and it will automatically generate and display the navigation links.

### Login Roles
The `ral_administrators` and `ral_users` modules provide user authentication and login functionality.  The two modules use the RocketAuthLogin crate, and each module defines two structures:
- A cookie struct for storing the user role in a cookie, named something like AdminCookie.  This will implement the `AuthorizeForm` trait.
- A form struct for storing and processing the role's login form data, named something like AdminForm.  This will implement the `AuthorizeCookie` trait.

The `AuthorizeForm` trait defines an authenticate method that is called to authenticate a user's credentials.  It also defines a new_form()



### Blog Module
The `blog` module contains the core data structures and methods like `Articles` and structs for processing them.  It also holds most of the support structs and query string data for things like full-text searching the database and sorting paginated content.

### Templates
There a several different types of templates for different types of pages, as well as some common partial templates for header and footer content.

The header partial template also uses the `assets-css` and `assets-js-before` while the footer uses `assets-js-after` partial templates which tell the header which versions of the css and js assets to use.


## Filesystem Layout
The blog uses a few folders for special purposes, the rest are just notes, or used by Cargo (like src and target).

- **logs* *** - holds json serialized log files to be loaded on startup, and saved frequently
- **pages** - all files in the pages folder are loaded as static pages, with the exception of .bak or .old files (which are skipped)
- **static** - these are public files which can be served from the root directory of the website, without needing to add the /static/ path, this is for security so files can not be served from the root project directory
- **templates** - these are handlebars (or tera) templates used for different types of pages and content
- **scripts** - holds executable scripts to be run from the administration panel.  Currently only the database backup script is ran from the website, but in the future the administration panel MAY add support for letting admins choose to run scripts that are in this directory

## Database Layout
The blog uses a Postgresql database that has two tables: `articles` and `users`.  The database also uses the pgcrypto extension for securely storing users' passwords.
The blog also defines a few Postgresql functions and triggers in the database.
- The `description()` function is used to only retrieve a shortened text instead of the entire field.  This function is run on from queries in the blog application whenever retrieving several articles and not needing the entire body.
- The `fulltxt_articles_update()` function is used to update the fulltext index on the articles table and is run whenever an article is update or inserted.
- The `proc_blog_users_insert()` function hashes a new users password and runs when a new user is inserted.
- The `proc_blog_users_update()` function hashes an existing user's new password and runs when a user table record is updated.

##### Articles Table
Most of the columns are relatively straight forward, but a few require a little explanation.

The `aid` column stores the ArticleID and uses the `oid` datatype, this is because the normal integer translates to the i64 datatype in Rust and the u32 is used for ArticleIDs, and the `oid` returns a u32.

The `tag` column stores an array of `character varying` strings, so to get all tags for the tagcloud the builtin `unnest()` function is needed.

The `fulltxt` column stores the fulltext data in a `tsvector`.
The `markdown` field stores the original markdown 
source used when creating or editing an article, which is processed into html and stored in the `body` column when saving.

##### Users Table
The `users` table stores the userid, username, display name, whether the user is an admin, their hashed password, and columns to prevent brute force attacks.

The `attempts` column tracks how many failed login attempts have been made since the user's last successful login.  This is not reset until successful login.

The `hash_salt` column is a text field that stores passwords using the `crypt()` funciton to securely hash the passwords using a random salt.  The `crypt()` function stores the salt and the hash in the same column, thus `hash_salt`.  When attempting to authenticate credentials the query uses the salt stored in the `hash_salt` column to `crypt()` the specified password.  This prevents the use of large databases of text and their corresponding hashes to lookup passwords.

Every so many failed login attempts triggers an account lockout.  The exact number of failed attempts before lockout depends on the settings file.  A lockout will prevent the user from loggin in for a certain period of time (again set in the settings file), telling them their account is locked.  The `lockout` column stores the timestamp of when they are able to try again, and any login attempts before the timestamp are denied.


