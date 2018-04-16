
#![feature(entry_and_modify)]
#![feature(custom_derive)]
#![feature(plugin)]
#![plugin(rocket_codegen)]
// #![plugin(dotenv_macros)]

#[macro_use] extern crate serde_json;
#[macro_use] extern crate serde_derive;
#[macro_use] extern crate lazy_static;
#[macro_use] extern crate indexmap;
extern crate rocket;
extern crate rocket_contrib;
extern crate serde;
extern crate serde_yaml;
extern crate rmp_serde as rmps;
extern crate chrono;
extern crate regex;
extern crate postgres;
extern crate r2d2;
extern crate r2d2_postgres;
extern crate libflate;
extern crate brotli;
extern crate zopfli;
extern crate comrak;
extern crate twoway;
extern crate chashmap;
extern crate evmap;
extern crate urlencoding;
extern crate titlecase;
extern crate rss;
#[allow(unused_imports)]
extern crate htmlescape;
extern crate rocket_auth_login;


mod cache;
mod routes;
mod counter;
mod content;
mod referrer;
mod location;
mod collate;
mod accept;
mod xpress;
mod layout;
mod blog;
mod data;
mod templates;
mod pages;
mod sanitize;
mod ral_administrator;
mod ral_user;

use routes::*;
use blog::*;
use cache::*;
use counter::*;
use xpress::*;
use accept::*;
use content::*;
use ral_administrator::*;
use data::*;
use pages::*;
use rocket_auth_login::authorization::*;
use rocket_auth_login::*;
use rocket_auth_login::sanitization::*;
use templates::*;



use regex::Regex;
use titlecase::titlecase;
use comrak::{markdown_to_html, ComrakOptions};
use indexmap::IndexMap;

use std::time::{Instant, Duration, SystemTime};
use std::ffi::OsStr;
use std::{env, str, io};
use std::io::{Cursor, Read};
use std::path::{Path, PathBuf};
use std::sync::{Mutex, Arc, RwLock};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::collections::HashMap;

use rocket_contrib::Template;
use rocket::{Request, Data, Outcome, Response};
use rocket::response::{NamedFile, Redirect, Flash, Responder, Content};
use rocket::response::content::Html;
use rocket::data::FromData;
use rocket::request::{FlashMessage, Form, FromForm, FormItems, FromRequest};
use rocket::http::{Cookie, Cookies, MediaType, ContentType, Status};
use rocket::State;


// Global settings are separated into a file called settings.rs
// This separation allows exclusion of the settings file from
//   things like git repos and other publicly viewable areas.
//   This allows passwords and server information to be kept
//   safe and secure while the rest of the project is uploaded
//   and can be viewed publicly.
include!("settings.rs");



#[get("/<file..>", rank=10)]
fn static_files(file: PathBuf, encoding: AcceptCompression) -> Option<Express> {
    if let Some(named) = NamedFile::open(Path::new("static/").join(file)).ok() {
        let exp: Express = named.into();
        Some( exp.compress(encoding).set_ttl(2592000isize) )
    } else {
        None
    }
}



#[error(404)]
pub fn error_not_found(req: &Request) -> Express {
    ErrorHits::error404(req);
    let content = format!( "The request page `{}` could not be found.", sanitize_text(req.uri().as_str()) );
    let output = hbs_template(TemplateBody::General(content), None, Some("404 Not Found.".to_string()), String::from("/404"), None, None, None, None);
    let express: Express = output.into();
    express
}
#[error(500)]
pub fn error_internal_error(req: &Request) -> Express {
    ErrorHits::error500(req);
    let content = format!( "An internal server error occurred procesing the page `{}`.", sanitize_text(req.uri().as_str()) );
    let output = hbs_template(TemplateBody::General(content), None, Some("500 Internal Error.".to_string()), String::from("/500"), None, None, None, None);
    let express: Express = output.into();
    express
}

lazy_static! {
    static ref PGCONN: Mutex<DbConn> = Mutex::new( DbConn(init_pg_pool().get().expect("Could not connect to database.")) );
}


fn main() {
    if PRODUCTION {
        println!("Production mode");
    } else {
        println!("Dev mode");
    }
    
    let mut pg_pool = data::init_pg_pool();
    let conn;
    if let Ok(pooled_conn) = pg_pool.get() {
        conn = DbConn(pooled_conn);
    } else {
        panic!("Connection could not be retrieved from db connection pool")
    }

    let hitcount: Counter = Counter::load();
    let views: TotalHits = TotalHits::load();
    let uhits: UStatsWrapper = UStatsWrapper( RwLock::new( UniqueStats::load() ) );
  
    let content_context: ContentContext = ContentContext::load(STATIC_PAGES_DIR);
    let content_cache: ContentCacheLock = ContentCacheLock::new();
    let article_map_cache = ArticleCacheLock::new( ArticleCache::load_cache(&conn) );
    let multi_aids = TagAidsLock::load_cache(&conn);
    let text_cache = TextCacheLock::new( TextCache::load_cache(&conn, &multi_aids) );
    let num_articles = NumArticles(AtomicUsize::new( article_map_cache.num_articles() as usize ));
    
    if CACHE_ENABLED {
        println!("Starting blog.  Cache enabled.");
    } else {
        println!("Starting blog.  Cache is DISABLED.");
    }
        
    let rock = rocket::ignite()
        .manage(pg_pool)
        .manage(hitcount)
        .manage(views)
        .manage(uhits)
        .manage(content_context)
        .manage(content_cache)
        .manage(article_map_cache)
        .manage(text_cache)
        .manage(multi_aids)
        .manage(num_articles)
        
        .attach(Template::fairing())
        
        .mount("/", routes![
            routes::articles::cache_index,
            routes::author::cache_author_seo,
            routes::author::cache_author,
            routes::rss::cache_rss,
            routes::tagcloud::cache_tagcloud,
            routes::tag::cache_tag_redirect,
            routes::tag::cache_tag,
            routes::article::cache_article_title,
            routes::article::cache_article_id,
            routes::article::cache_article_view,
            routes::article::hbs_article_not_found,

            pages::misc::static_pages,
            pages::misc::code_download,
            pages::manage::create::hbs_article_process,
            pages::manage::create::hbs_create_unauthorized,
            pages::manage::create::hbs_create_form,
            pages::manage::edit::hbs_edit,
            pages::manage::edit::hbs_edit_process,
            pages::manage::delete::hbs_delete_confirm,
            pages::manage::delete::hbs_process_delete,
            pages::search::hbs_search_redirect,
            pages::search::hbs_search_results,
            pages::website::hbs_about,
            pages::manage::dashboard::hbs_manage_basic,
            pages::manage::dashboard::hbs_manage_full,
            pages::roles::admins::hbs_dashboard_admin_authorized,
            pages::roles::admins::hbs_dashboard_admin_flash,
            pages::roles::admins::hbs_dashboard_admin_retry_user,
            pages::roles::admins::hbs_process_admin_login,
            pages::roles::admins::hbs_logout_admin,
            pages::manage::misc::backup,
            pages::roles::admins::hbs_dashboard_admin_retry_redir,
            pages::roles::admins::hbs_dashboard_admin_retry_redir_only,
            pages::roles::regular::hbs_dashboard_user_authorized,
            pages::roles::regular::hbs_dashboard_user_retry_user,
            pages::roles::regular::hbs_dashboard_user_flash,
            pages::roles::regular::hbs_process_user_login,
            pages::roles::regular::hbs_logout_user,
            pages::manage::misc::refresh_content,
            pages::manage::stats::hbs_pagestats,
            pages::manage::stats::hbs_pagestats_unauthorized,
            pages::manage::stats::hbs_pagestats_no_errors,
            pages::manage::stats::hbs_pageviews,
            pages::manage::edit::hbs_edit_unauthorized,
            pages::manage::dashboard::hbs_manage_unauthorized,
            pages::manage::delete::hbs_delete_unauthorized,
            pages::manage::stats::hbs_pageviews_unauthorized,
            pages::manage::misc::hbs_backup_unauthorized,
            
            static_files
        ])
        .catch(errors![ error_internal_error, error_not_found ])
        .launch();
}


