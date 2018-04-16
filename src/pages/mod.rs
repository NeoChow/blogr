
use rocket_contrib::Template;
use rocket::response::{content, NamedFile, Redirect, Flash};
use rocket::{Request, Data, Outcome};
use rocket::request::{FlashMessage, Form, FromForm};
use rocket::data::FromData;
use rocket::response::content::Html;
use rocket::State;
use rocket::http::{Cookie, Cookies, RawStr};
use rocket::http::hyper::header::{Headers, ContentDisposition, DispositionType, DispositionParam, Charset};

use std::{thread, time};
use std::time::Instant;
use std::time::Duration;
use std::{env, str, io};
use std::io::{Cursor, Read};
use std::path::{Path, PathBuf};
use std::io::prelude::*;
use std::fs::{self, File};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Mutex, Arc, RwLock};

use regex::Regex;
use titlecase::titlecase;
use chrono::prelude::*;
use chrono::{NaiveDate, NaiveDateTime};
use comrak::{markdown_to_html, ComrakOptions};

use super::*;
use cache::*;
use content::{destruct_cache, destruct_context, destruct_multi};
use cache::body::*;
use cache::pages::*;
use counter::*;
use location::*;
use referrer::*;
use collate::*;
use layout::*;
use blog::*;
use data::*;
use sanitize::*;
use rocket_auth_login::authorization::*;
use rocket_auth_login::sanitization::*;
use ral_administrator::*;
use ral_user::*;
use templates::*;
use xpress::*;
use accept::*;


pub mod misc;
pub mod search;

pub mod roles {
    use super::*;
    pub mod admins;
    pub mod regular;
}

pub mod manage {
    use super::*;
    pub mod create;
    pub mod edit;
    pub mod dashboard;
    pub mod delete;
    pub mod misc;
    pub mod stats;
}

pub mod website {
    use super::*;
    
    #[get("/about")]
    pub fn hbs_about(start: GenTimer, conn: DbConn, admin: Option<AdministratorCookie>, user: Option<UserCookie>, encoding: AcceptCompression, uhits: UniqueHits) -> Express {
        let output = hbs_template(TemplateBody::General("This page is not implemented yet.  Soon it will tell a little about me.".to_string()), None, Some("About Me".to_string()), String::from("/about"), admin, user, None, Some(start.0));
        let express: Express = output.into();
        express.compress(encoding)
    }
}


