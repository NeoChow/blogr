# Blogr Design

## Requirements
The blog application is written in the Rust programming language and uses the Rocket web framework.  It currently uses a Postgresql database, which should become optional in the future along with the option to use other databases.

### Operation Modes
It has a default dev mode and a production mode which is activated by using rustc's --cfg like: `cargo rustc --release -- --cfg production`.

### Servers
I use nginx to host the blog, but it is not dependent upon a specific server.  In a typical reverse proxy setup it is recommended that the ip address is forwarded, allowing the true ip address of the client to be discovered by the application instead of only the loopback address.

I have found using servers such as Apache or Nginx beneficial, as the web server built into the generated Rust binary is a much more basic server, with less features.  The reverse proxy setup allows all requests to first go to the Apache/Nginx/etc server where static files like css/js/images can be served directly to the client, and all other requests are forwarded to the Rust web app which will generate a response which will be sent back to the user (through the Apache/Nginx/etc server).

It is highly recommended to enable TLS (SSL).  This is actually fairly easy to do, and the hardest part is installing the software to generate the certificate.  The rocket-auth-login crate has examples which detail an SSL setup further.

## Modules
The xpress module defines an Express struct which is used by all route functions to return data which will be sent to the client, along with being able to add expiration headers, compression, and other response options/headers (including force downloading the response).

The routes module defines the actual routes that are registered in Rocket.  These functions are usually very short and call other functions to do most of the actual processing.  The cache::pages submodule, the pages module, and the page module's submodules contain most of the code executed for most of the routes.

The content module loads files from a folder to generate a cache of pages.  These content pages are not article pages, they are intended to be used by having articles link to non-article pages as support pages.  For example, you would use a content page when an article wants to link to some further content, directions, or information that is outside the scope of the article but related, and would not be appropriate to post in an article (guides, help information, about the author or website, etc).  These files are cached wholly, their html and compressed versions are cached, compared to articles which only cache the article's data and require the Handlebars template to generate the page on each request (but allowing menus to show menu items based on whether the user is logged in or is an admin, which cannot be done with the content files very easily).

The blog module contains many data structures and implementations related to the blog itself, like article data structures and helper data structures for forms.

The data module helps connect to and interact with the database and defines a DbConn type and several methods to help make querying the database easier.  The DbConn has a deref implementation that returns a Connection type.

The collate module's purpose is to split up pages consisting of many articles into several pages containing a few articles.  This module is both complicated and a little bit magical.  It primarily handles finding out which page is being viewed and displaying links to the other pages, based on settings defined in a separate struct (implementing the Collate trait which defines methods that return the various settings).

The ral_administrator and ral_user modules use the rocket-auth-login crate to define different types of users.  The ral_administrator module defines, among other structures, the AdminisratorCookie type which is specified as a parameter in the routes requiring the client to be an administrator (or Option<AdministratorCookie> when the client does not need to be an administrator).  The ral_user defines a regular user type, and could be used for making comments, for example.

The accept module is used to determine what type of compression algorithms (encodings) the client accepts, and will determine the best compression method to use given the available methods.

The counter module tracks statistics on page views.  It counts the number of times a unique visitor has viewed a page, along with the total number of page views for the website.  When the application is run for the first time it will initialize the various statistic data structures, serialize them, and save them in files inside the log folder.  After the first time running the application will instead load the serialized log data to continue where it left off.  The settings.rs file defines an interval in which the log data is serialized, and if the application crashes or is shutdown the log data for the clients within that inverval may not be saved.

The layout module defines various functions (and a few constants) to generate HTML for specific page items (such as alert messages and navigation HTML).  It was initially used to generate all HTML and data sent to the client, but the majority of the code was removed and replaced with the template and xpress modules.

The template module interacts with the Handlebars templates, taking page information and generating HTML output by rendering the associated Handlebars template with its context (page information).

The location module is a very small module which defines a data structure that determines the uri (the location) of the requested page.  It also has a function to create a link to the admin login page with the currently requested page as a referer.

The referer module is another very small module, and contains a data structure that determines the page that refered the client to the current page (the refering page, or previously visited page).

The settings.rs file specifies several global constants which can be edited to change the behavior of the application.  Each blog install will use settings to correctly setup the application and determine various limits, filenames, folders, menu items, markdown configuration, and more.  The settings.rs file is included into the main.rs by the include!() macro.

The private module is an extension of the settings.rs file.  It contains database username and password information, and is excluded from the git repository.

The main.rs file pulls in the settings, defines some error handling routes (soon to be replaced by the newer Catcher in Rocket 0.3.15), creates a shared database connection pool, intializes the various caches, and load files.  The main() function also intializes the Rocket instance, which will start managing the various caches and state as well as mount the available routes for the blog, and finally launch the app.

## Folders

The database folder mostly contains sql files for learning and documentation purposes.

The logs folder is an essential folder containing all of the log files which track statistics like page hits and unique users.

The notes folder can be ignored and only contains text documents with various notes in them.

The pages folder is a special folder that holds all of the non-article content for the blog website.  Each file is a separate page, which are usually used as support pages for articles to link to, or content that you can link to in the menus but should not appear in the article listings.

The scripts folder is currently used to hold scripts executed on demand in the administrator control panel (currently only the database backup script), and scripts executed on the server by a local user, not by the web app.

The static folder contains all of the statically served content like js/css/fonts/image files.  This folder will also hold uploaded content when that feature is added (for now the content can be uploaded manually to the upload directory).  Most files in the static folder can be served by Apache/Nginx servers, instead of the blog app, by setting up the Apache/Nginx server to serve those files.  If the blog app is not used to serve the static files it is recommended that a copy of the files reside in this folder anyways so that if the Apache/Nginx server cannot serve the file (or it is corrupted) the blog app will be used as a fallback to serve the files, as any request that is not caught by a predefined route in Rocket will attempt to find and securely serve the requested file from the static folder.

The templates folder defines the layout of the website in several Handlebar templates and partial templates.  The files in the common folder are partial templates used to define the header, footer, and css/js file versions (they use versioning to prevent old cached versions of the files from being used after a new updated file is introduced).  The templates directly inside the templates folder define various page types, like displaying a single article, multiple articles, content pages, various management pages, and more.

Notable files included in the root folder are the Rocket.toml file (used to configure Rocket and enable SSL), and the license file.  It also currently contains a blog-organization.md file which was the first attempt at defining and documenting the design and organization of the blog application, and still contains useful and valid information.






