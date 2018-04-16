
use rocket_contrib::Template;
use rocket::{Request, Data, Outcome, Response};
use rocket::response::{NamedFile, Redirect, Flash, Responder, Content};
use rocket::response::content::Html;
use rocket::data::FromData;
use rocket::request::{FlashMessage, Form, FromForm, FormItems, FromRequest};
use rocket::http::{Cookie, Cookies, MediaType, ContentType, Status};
use rocket::State;

use std::fmt::Display;
use std::{env, str, thread};
use std::fs::{self, File, DirEntry};
use std::io::prelude::*;
use std::io::{self, Cursor, Read};
use std::path::{Path, PathBuf};
use std::time::{self, Instant, Duration};
use std::prelude::*;
use std::ffi::OsStr;
use std::collections::HashMap;
use std::sync::{Mutex, Arc, RwLock};
use std::sync::atomic::AtomicUsize;

use std::borrow::Cow;

use evmap::*;
use comrak::{markdown_to_html, ComrakOptions};

use super::super::*;
use super::*;
use ::blog::*;
use ::data::*;
use ::content::*;
use ::templates::*;
use ::xpress::*;
use ::ral_user::*;
use ::ral_administrator::*;


pub trait BodyContext {
    fn template_name() -> &'static str;
}

#[derive(Debug, Clone, Serialize)]
pub struct CtxBody<T: BodyContext>(pub T);

impl BodyContext for TemplateArticlesPages { fn template_name() -> &'static str { "articles-pagination-template"} }
impl BodyContext for TemplateGeneral { fn template_name() -> &'static str { "general-template"} }
impl BodyContext for TemplateArticle { fn template_name() -> &'static str { "article-template"} }
impl BodyContext for TemplateTags { fn template_name() -> &'static str { "tags-template"} }

// Admin pages should not be cached so the structs below should not
// need to be used, they may but they are not needed (most likely)
impl BodyContext for TemplateLogin { fn template_name() -> &'static str { "login-template"} }
impl BodyContext for TemplateLoginData { fn template_name() -> &'static str { "login-template"} }
impl BodyContext for TemplateCreate { fn template_name() -> &'static str { "create-template"} }
impl BodyContext for TemplateEdit { fn template_name() -> &'static str { "edit-article-template"} }
impl BodyContext for TemplateSearch { fn template_name() -> &'static str { "search-template"} }
impl BodyContext for TemplateManage { fn template_name() -> &'static str { "manage-pagination-template"} }
impl BodyContext for TemplateArticles  { fn template_name() -> &'static str { "articles-template"} }// Is this still used??



