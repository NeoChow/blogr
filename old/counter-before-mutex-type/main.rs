
/* Todo:
  JOIN
    Add a signup page
    
  COMMENTS
    Add comments for logged in users
      -Users that are not logged in will just see a link to 
          Login to Comment instead of a post new comment form
      -List all comments below the article
      -Allow admin users to delete comments
      -Routes: Admin Users: can create comments and delete comments
               Regular users: can create comments and delete own comments
               Public: can view comments and see a link to login to post comments   

*/


#![feature(custom_derive)]
#![feature(plugin)]
#![plugin(rocket_codegen)]

// extern crate multipart;
extern crate rocket;
extern crate rocket_contrib;
// extern crate rocket_simpleauth as auth;
extern crate serde;
#[macro_use] extern crate serde_json;
#[macro_use] extern crate serde_derive;
extern crate rmp_serde as rmps;

extern crate chrono;
#[macro_use] extern crate lazy_static;
extern crate regex;
// extern crate time;

extern crate postgres;
extern crate r2d2;
extern crate r2d2_postgres;

// extern crate rocket_file_cache;
// extern crate concurrent_hashmap;
// extern crate chashmap;
extern crate markdown;

extern crate libflate;
extern crate brotli;
extern crate zopfli;

extern crate urlencoding;
extern crate titlecase;
// extern crate handlebars;
#[allow(unused_imports)]
extern crate htmlescape;
extern crate rss;
extern crate dotenv;
// #[macro_use] extern crate log;
// extern crate env_logger;
// #[macro_use] extern crate diesel_codegen;
// #[macro_use] extern crate diesel;

extern crate rocket_auth_login;

// mod vcache;
mod counter;
mod collate;
mod accept;
mod xpress;
mod layout;
mod blog;
mod data;
mod hbs_templates;
mod pages;
mod sanitize;
mod ral_administrator;
mod ral_user;
mod pages_administrator;
// mod templates;
// mod cookie_data;
// mod admin_auth;
// mod user_auth;
// mod users;
// mod login_form_status;

// use cache::*;
// use vcache::*;
use counter::*;
use xpress::*;
use accept::*;
use ral_administrator::*;
use pages_administrator::*;
use data::*;
use pages::*;
// use templates::TemplateMenu;
use rocket_auth_login::authorization::*;
use rocket_auth_login::*;
use rocket_auth_login::sanitization::*;
use hbs_templates::*;

// use handlebars::Handlebars;
use titlecase::titlecase;
use regex::Regex;
use std::time::{Instant, Duration, SystemTime};

use std::ffi::OsStr;
use std::{env, str, io};
use std::io::{Cursor, Read};
use std::path::{Path, PathBuf};
use std::sync::{Mutex, Arc};
use std::collections::HashMap;

// use chashmap::*;

// Rocket File Cache - Removed
// use rocket_file_cache::{Cache, CachedFile};
// use std::sync::Mutex;
// use std::path::{Path, PathBuf};
// use rocket::State;

// use concurrent_hashmap::*;

use rocket_contrib::Template;
use rocket::{Request, Data, Outcome, Response};
use rocket::response::{content, NamedFile, Redirect, Flash, Responder, Content};
use rocket::response::content::Html;
use rocket::data::FromData;
use rocket::request::{FlashMessage, Form, FromForm, FormItems, FromRequest};
use rocket::http::{Cookie, Cookies, MediaType, ContentType, Status};
use rocket::State;


// use auth::userpass::UserPass;
// use auth::status::{LoginStatus,LoginRedirect};
// use auth::dummy::DummyAuthenticator;
// use auth::authenticator::Authenticator;

// use chrono::prelude::*;
// use multipart::server::Multipart;



// BLOG_URL MUST HAVE A TRAILING FORWARD SLASH /
pub const BLOG_URL: &'static str = "http://localhost:8000/";
pub const USER_LOGIN_URL: &'static str = "http://localhost:8000/user";
pub const ADMIN_LOGIN_URL: &'static str = "http://localhost:8000/admin";
pub const TEST_LOGIN_URL: &'static str = "http://localhost:8000/login";
pub const CREATE_FORM_URL: &'static str = "http://localhost:8000/create";
pub const EDIT_FORM_URL: &'static str = "http://localhost:8000/edit";
pub const MANAGE_URL: &'static str = "http://localhost:8000/manage";
pub const MAX_CREATE_TITLE: usize = 120;
pub const MAX_CREATE_DESCRIPTION: usize = 400;
pub const MAX_CREATE_TAGS: usize = 250;










#[get("/<file..>", rank=10)]
fn static_files(file: PathBuf, encoding: AcceptCompression) -> Option<Express> {
    // if let Some(named) = NamedFile::open(Path::new("static/").join(file)).ok() {
    if let Some(named) = NamedFile::open(Path::new("static/").join(file)).ok() {
        let exp: Express = named.into();
        Some( exp.compress(encoding) )
    } else {
        None
    }
}



// #[get("/<file..>", rank=10)]
// fn static_files(file: PathBuf, encoding: AcceptCompression) -> Option<Express> {
//     // if let Some(named) = NamedFile::open(Path::new("static/").join(file)).ok() {
//     if let Some(named) = NamedFile::open(Path::new("static/").join(file)).ok() {
//         let cached = CachedFile::open(named.path(), cache.inner());
        
//         // let exp: Express = named.into();
//         let exp: Express = cached.into();
//         Some( exp.comprses(encoding) )
//     } else {
//         None
//     }
// }


// }
// #[get("/<file..>")]
// fn files(file: PathBuf, cache: State<Cache> ) -> Option<CachedFile> {
//     CachedFile::open(Path::new("www/").join(file), cache.inner())
// }

// #[get("/<file..>", rank=10)]
// // fn static_files(file: PathBuf, encoding: AcceptCompression) -> Option<Express> {
// fn static_files(file: PathBuf, vcache: State<VCache>, encoding: AcceptCompression) -> Option<Express> {
// // fn static_files(file: PathBuf) -> Option<NamedFile> {
//     // Without Expiration header:
//     // NamedFile::open(Path::new("static/").join(file)).ok()
//     let start = Instant::now();
//     if let Some(named) = NamedFile::open(Path::new("static/").join(file)).ok() {
//         let result: Option<Express>;
//         if let Some(output) = VCache::retrieve(named.path().to_path_buf(), vcache) {
//             // let express: Express = output.into();
            
//             let ctype = ContentType::from_extension(named.path().extension().unwrap_or(OsStr::new("")).to_str().unwrap_or("")).unwrap_or(ContentType::Plain);
//             let express: Express = output.into();
//             // express.compress(encoding)
//             result = Some( express.set_content_type(ctype).compress(encoding) );
            
//         } else {
//             println!("Cache retrieve failed, falling back to named file");
//             let exp: Express = named.into();
//             result = Some( exp.compress(encoding) );
//         }
        
//         let end = start.elapsed();
//         println!("Served static file in {}.{:08} seconds", end.as_secs(), end.subsec_nanos());
        
//         result
//         // let exp: Express = named.into();
//         // Some( exp )
//     } else {
//         None
//     }
    
// }


#[error(404)]
pub fn error_not_found(req: &Request) -> Express {
    let content = format!( "The request page `{}` could not be found.", sanitize_text(req.uri().as_str()) );
    let output = hbs_template(TemplateBody::General(content, None), Some("404 Not Found.".to_string()), String::from("/404"), None, None, None, None);
    let express: Express = output.into();
    express
}
#[error(500)]
pub fn error_internal_error(req: &Request) -> Express {
    let content = format!( "An internal server error occurred procesing the page `{}`.", sanitize_text(req.uri().as_str()) );
    let output = hbs_template(TemplateBody::General(content, None), Some("500 Internal Error.".to_string()), String::from("/500"), None, None, None, None);
    let express: Express = output.into();
    express
}

lazy_static! {
    static ref PGCONN: Mutex<DbConn> = Mutex::new( DbConn(init_pg_pool().get().expect("Could not connect to database.")) );
}


fn main() {
    // env_logger::init();
    // init_pg_pool();
    // if let Ok(pg_conn) = init_pg_pool().get() {
    //     pg_conn;
    //     // (*pg_conn).connect();
    // }
    
    // let hitcount: PageCount = PageCount::new();
    // let views: ViewsTotal = ViewsTotal::new();
    let hitcount: PageCount = PageCount::load();
    let views: ViewsTotal = ViewsTotal::load();
    
    
    // let vcache: VCache = VCache(CHashMap::new());
    
    init_pg_pool().get().unwrap();
    
    rocket::ignite()
        // .manage(vcache)
        .manage(data::init_pg_pool())
        .manage(hitcount)
        .manage(views)
        .attach(Template::fairing())
        .mount("/", routes![
            
            // pages::hbs_admin_page,
            // pages::hbs_admin_login,
            // pages::hbs_admin_retry,
            // pages::hbs_process_admin,
            // pages::hbs_user_page,
            // pages::hbs_user_login,
            // pages::hbs_user_retry,
            // pages::hbs_user_process,
            
            pages::pagination_test,
            
            // pages::hbs_all_articles,
            // pages::hbs_articles_page,
            pages::hbs_view_articles,
            pages::hbs_tags_all,
            pages::hbs_articles_tag_redirect,
            pages::hbs_articles_tag,
            pages::hbs_article_title,
            pages::hbs_article_id,
            pages::hbs_article_view,
            pages::hbs_article_not_found,
            pages::hbs_article_process,
            pages::hbs_create_unauthorized,
            pages::hbs_create_form,
            pages::hbs_edit,
            pages::hbs_edit_process,
            // pages::hbs_manage,
            
            // pages::hbs_logout,
            pages::hbs_search_page,
            pages::hbs_search_redirect,
            pages::hbs_search_results,
            pages::hbs_author_display,
            pages::hbs_author,
            pages::hbs_about,
            pages::rss_page,
            pages::hbs_index,
            
            pages::hbs_manage_basic,
            pages::hbs_manage_full,
            
            // pages_administrator::dashboard_admin_unauthorized,
            pages::hbs_dashboard_admin_authorized,
            pages::hbs_dashboard_admin_flash,
            pages::hbs_dashboard_admin_retry_user,
            // pages_administrator::dashboard_admin_retry_flash,
            pages::hbs_process_admin_login,
            pages::hbs_logout_admin,
            
            pages::hbs_dashboard_user_authorized,
            // pages_administrator::dashboard_user_unauthorized,
            // pages::dashboard_user_login,
            pages::hbs_dashboard_user_retry_user,
            pages::hbs_dashboard_user_flash,
            // pages_administrator::dashboard_user_retry_flash,
            pages::hbs_process_user_login,
            pages::hbs_logout_user,
            
            pages_administrator::resp_test,
            pages_administrator::uncompressed,
            pages_administrator::compress_test,
            pages_administrator::compress_test2,
            pages_administrator::compress_test3,
            pages_administrator::compress_test4,
            pages_administrator::compress_gzip,
            pages_administrator::compress_deflate,
            pages_administrator::compress_brotli,
            
            pages::hit_count,
            pages::hit_count2,
            pages::hit_count3,
            
            static_files
        ])
        .catch(errors![ error_internal_error, error_not_found ])
        .launch();
}

