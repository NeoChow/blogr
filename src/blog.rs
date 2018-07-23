
use rocket;
use ::rocket::Request;
use ::rocket::request::{FromRequest, FromForm, FormItems, FromFormValue, FromParam};
use ::rocket::outcome::Outcome;
use ::rocket::config::{Config, Environment};
use rocket::http::RawStr;

use std::fmt::Display;
use std::time::Instant;
use postgres::{Connection};
use regex::Regex;
use evmap::*;
use titlecase::titlecase;
use htmlescape::*;
use chrono::prelude::*;
use chrono::{NaiveDate, NaiveDateTime};
use rocket_auth_login::sanitization;

use super::{BLOG_URL, MAX_CREATE_TITLE, MAX_CREATE_DESCRIPTION, MAX_CREATE_TAGS, DESC_LIMIT};

use data::*;
use sanitize::*;



/// ArticleId is used to retrieve articles from an Article ID
#[derive(Debug, Clone)]
pub struct ArticleId {
    pub aid: u32,
}

// used for retrieving a GET url tag
#[derive(Debug, Clone)]
pub struct Tag {
    pub tag: String,
}

#[derive(Debug, Clone)]
pub struct Article {
    pub aid: u32,
    pub title: String,
    pub posted: NaiveDateTime,
    pub modified: NaiveDateTime,
    pub userid: u32,
    pub username: String,
    pub body: String,
    pub tags: Vec<String>,
    pub description: String,
    pub markdown: String,
    pub image: String,
}

/// ArticleWrapper is used to retrieve existing article info from a form.
/// The /edit route is the only page module route this struct is used in
#[derive(Debug, Clone)]
pub struct ArticleWrapper {
    pub aid: u32,
    pub title: String,
    pub posted: NaiveDateTimeWrapper,
    pub modified: NaiveDateTimeWrapper,
    pub userid: u32,
    pub username: String,
    pub body: String,
    pub tags: String,
    pub description: String,
    pub markdown: String,
    pub image: String,
}


/// ArticleDisplay is passed to the handlebars templates' context.
#[derive(Debug, Clone, Serialize)]
pub struct ArticleDisplay {
    pub aid: u32,
    pub title: String,
    pub posted_machine: String,
    pub posted_human: String,
    pub modified_machine: String,
    pub modified_human: String,
    pub userid: u32,
    pub username: String,
    pub body: String,
    pub tags: Vec<String>,
    pub description: String,
    pub markdown: String,
    pub image: String,
}


/// Used for creating a new article
#[derive(Debug, Clone)]
pub struct ArticleForm {
    // pub userid: u32,
    pub title: String,
    pub body: String,
    pub markdown: String,
    pub tags: String,
    pub description: String,
    pub image: String,
}

/// Holds search submission information.  Converts form data into this struct.
#[derive(Debug, Clone)]
pub struct Search {
    pub limit: Option<u16>, // use u16 as limit as u16 does not implement FromSql
    pub o: Option<String>, // opposite / negated
    pub p: Option<String>, // possible words, or'd
    pub q: Option<String>, // query, and'd together
    pub min: Option<NaiveDateTimeWrapper>, // min
    pub max: Option<NaiveDateTimeWrapper>, // min
}

/// Structure to pass as context to handlebars template.
#[derive(Serialize)]
pub struct SearchDisplay {
    pub limit: u16,
    pub q: String,
    pub min: String,
    pub max: String,
}

/* 
/// Current page and number of article per page.  Deprecated
#[derive(FromForm)]
pub struct ViewPage {
    pub page: u32,
    // Articles Per Page
    pub app: Option<u8>,
}
 */

pub struct ArticleSearch {
    pub keywords: Vec<String>,
}

/// User information
#[derive(Debug, Clone)]
pub struct User {
    pub userid: u32,
    pub username: String,
    pub display: Option<String>,
    pub email: Option<String>,
    pub password: String,
    pub is_admin: bool,
    pub is_public: bool,
}

/// A structure used for timing how long page generation takes.
#[derive(Debug, Clone)]
pub struct GenTimer (pub Instant);

/// A wrapper around NaiveDateTime to allow datetimes to be read from requests.
#[derive(Debug, Clone)]
pub struct NaiveDateTimeWrapper(pub NaiveDateTime);


/// Used for the Sort enum.
pub type Descending = bool;

/// Holds current sort settings.
#[derive(Debug, Clone, Serialize)]
pub enum Sort {
    Title(Descending),
    Date(Descending),
}

/// Sort settings to be passed as context to handlebars templates.
#[derive(Debug, Clone, Serialize)]
pub struct SortDisplay {
    sort_title: bool,
    sort_date: bool,
    sort_desc: bool,
    sort_asc: bool,
}


/// URL query string containing a username
#[derive(Debug, Clone, )]
pub struct QueryUser {
    pub user: String,
}

/// URL query string containing a username and referring uri
#[derive(Debug, Clone, )]
pub struct QueryUserRedir {
    pub user: String,
    pub referrer: String,
}

/// URL query string containing a referring uri
#[derive(Debug, Clone, )]
pub struct QueryRedir {
    pub referrer: String,
}

impl<'f> FromForm<'f> for QueryUser {
    type Error = &'static str;
    fn from_form(form_items: &mut FormItems<'f>, _strict: bool) -> Result<Self, Self::Error> {
        let mut user: String = String::new();
        
        for (field, value) in form_items {
            match field.as_str() {
                "user" => { user = value.url_decode().unwrap_or( String::new() ); },
                _ => {},
            }
        }
        
        if &user != "" {
            Ok( QueryUser {
                user,
            } )
        } else {
            println!("QueryRedir is not valid.  user: {}", user);
            Err( "There was a missing field in QueryUser" )
        }
    }
}

impl<'f> FromForm<'f> for QueryUserRedir {
    type Error = &'static str;
    fn from_form(form_items: &mut FormItems<'f>, _strict: bool) -> Result<Self, Self::Error> {
        let mut user: String = String::new();
        let mut referrer: String = String::new();
        
        for (field, value) in form_items {
            match field.as_str() {
                "user" => { user = value.url_decode().unwrap_or( String::new() ); },
                "referrer" | "redir" | "redirect" => { referrer = value.url_decode().unwrap_or( String::new() ); },
                _ => {},
            }
        }
        if &user != "" && &referrer != "" {
            Ok( QueryUserRedir {
                user,
                referrer,
            } )
        } else {
            println!("QueryRedir is not valid.  user: {}, referrer: {}", user, referrer);
            Err( "There was a missing field in QueryUserRedir" )
        }
    }
}

impl<'f> FromForm<'f> for QueryRedir {
    type Error = &'static str;
    fn from_form(form_items: &mut FormItems<'f>, _strict: bool) -> Result<Self, Self::Error> {
        let mut referrer: String = String::new();
        
        for (field, value) in form_items {
            match field.as_str() {
                "referrer" | "redir" | "redirect" => { referrer = value.url_decode().unwrap_or( String::new() ); },
                _ => {},
            }
        }
        
        if &referrer != "" {
            Ok( QueryRedir {
                referrer,
            } )
        } else {
            println!("QueryRedir is not valid.  referrer: {}", referrer);
            Err( "There was a missing field in QueryRedir" )
        }
    }
}



pub fn now() -> NaiveDateTime {
    Local::now().naive_local()
}

pub fn opt_col<T>(rst: Option<Result<T, T>>) -> T where T: Display + Default {
    match rst {
        Some(Ok(d)) => d,
        Some(Err(e)) => { println!("Encountered an error retrieving the description. Error: {}", e); T::default() },
        None => T::default(),
    }
}


impl Sort {
    pub fn to_display(&self) -> SortDisplay {
        match self {
            &Sort::Title(desc) if desc == true => SortDisplay { sort_title: true, sort_date: false, sort_desc: true, sort_asc: false },
            &Sort::Title(desc) => SortDisplay { sort_title: true, sort_date: false, sort_desc: false, sort_asc: true },
            &Sort::Date(desc) if desc == true => SortDisplay { sort_title: false, sort_date: true, sort_desc: true, sort_asc: false },
            &Sort::Date(desc) => SortDisplay { sort_title: false, sort_date: true, sort_desc: false, sort_asc: true },
            _ => SortDisplay { sort_title: true, sort_date: false, sort_desc: true, sort_asc: false },
        }
    }
}

impl SearchDisplay {
    pub fn default() -> SearchDisplay {
        SearchDisplay {
            limit: 0,
            q: String::new(),
            min: String::new(),
            max: String::new(),
        }
    }
}

impl Search {
        pub fn to_query(&self) -> String {
        let mut output: String = String::with_capacity(120);
        
        let mut empty = true;
        
        if let Some(ref q) = self.q {
            if empty {
                output.push_str("q=");
                output.push_str(&q);
            } else {
                output.push_str("&q=");
                output.push_str(&q);
                empty = false;
            }
        }
        if let Some(ref min) = self.min {
            if empty {
                output.push_str("min=");
                output.push_str( &format!("{}", min.0.format("%Y-%m-%d %H:%M:%S")) );
            } else {
                output.push_str("&min=");
                output.push_str( &format!("{}", min.0.format("%Y-%m-%d %H:%M:%S")) );
                empty = false;
            }
        }
        if let Some(ref max) = self.max {
            if empty {
                output.push_str("max=");
                output.push_str( &format!("{}", max.0.format("%Y-%m-%d %H:%M:%S")) );
            } else {
                output.push_str("&max=");
                output.push_str( &format!("{}", max.0.format("%Y-%m-%d %H:%M:%S")) );
                empty = false;
            }
        }
        output
    }
    pub fn to_display(&self) -> SearchDisplay {
        SearchDisplay {
            limit: if let Some(limit) = self.limit { limit } else { 0 },
            q: if let Some(ref q) = self.q { q.to_string() } else { String::new() },
            min: if let Some(ref min) = self.min { format!("{}", min.0.format("%Y-%m-%d %H:%M:%S")) } else { String::new() },
            max: if let Some(ref max) = self.max { format!("{}", max.0.format("%Y-%m-%d %H:%M:%S")) } else { String::new() },
        }
    }
    pub fn default() -> Search {
        Search {
            limit: None,
            o: None,
            p: None,
            q: None,
            min: None,
            max: None,
        }
    }
}

impl ArticleWrapper {
    pub fn to_article(self) -> Article {
        Article {
            aid: self.aid,
            title: self.title,
            posted: self.posted.0.clone(),
            userid: self.userid,
            username: self.username,
            body: self.body,
            tags: split_tags(self.tags),
            description: self.description,
            markdown: self.markdown,
            image: self.image,
            modified: self.posted.0,
        }
    }
    pub fn is_valid(&self) -> bool {
        self.aid != 0
        && &self.title != ""
        && self.userid != 0
    }
}

impl<'a, 'r> FromRequest<'a, 'r> for GenTimer {
    type Error = ();
    
    fn from_request(request: &'a Request<'r>) -> ::rocket::request::Outcome<GenTimer,Self::Error> {
        Outcome::Success( GenTimer( Instant::now() ) )
    }
}



impl ArticleId {
    pub fn new(aid: u32) -> ArticleId {
        ArticleId {
            aid
        }
    }
    pub fn exists(&self) -> bool {
        unimplemented!()
    }
    // Retrieve with a new connection - not a pooled connection
    //   Do not use unless you have to - unless you have no db connection
    pub fn retrieve(&self) -> Option<Article> {
        let pgconn = establish_connection();
        let rawqry = pgconn.query(&format!("SELECT a.aid, a.title, a.posted, a.body, a.tag, a.description, u.userid, u.display, u.username, a.image, a.markdown, a.modified FROM articles a JOIN users u ON (a.author = u.userid) WHERE a.aid = {id}", id=self.aid), &[]);
        if let Ok(aqry) = rawqry {
            // println!("Querying articles: found {} rows", aqry.len());
            if !aqry.is_empty() && aqry.len() == 1 {
                let row = aqry.get(0); // get first row
                
                let display: Option<String> = row.get(7);
                let username: String = if let Some(disp) = display { disp } else { row.get(8) };
                let image: String = row.get_opt(9).unwrap_or(Ok(String::new())).unwrap_or(String::new());
                
                
                Some( Article {
                    aid: row.get(0),
                    title: row.get(1), // todo: call sanitize title here
                    posted: row.get(2),
                    body: row.get(3), // Todo: call sanitize body here
                    tags: row.get_opt(4).unwrap_or(Ok(Vec::<String>::new())).unwrap_or(Vec::<String>::new()).into_iter().map(|s| s.trim_matches('\'').trim().to_string()).filter(|s| s.as_str() != "").collect(),
                    description: row.get_opt(5).unwrap_or(Ok(String::new())).unwrap_or(String::new()),
                    userid: row.get(6),
                    username: titlecase( &sanitization::sanitize(&username) ),
                    markdown: row.get_opt(10).unwrap_or(Ok(String::new())).unwrap_or(String::new()),
                    image,
                    modified: row.get(11),
                })
            } else { None }
        } else { None }
    }
    // Prefer to use this over retrieve()
    pub fn retrieve_with_conn(&self, pgconn: DbConn) -> Option<Article> {
        let qrystr = format!("SELECT a.aid, a.title, a.posted, a.body, a.tag, a.description, u.userid, u.display, u.username, a.image, a.markdown, a.modified FROM articles a JOIN users u ON (a.author = u.userid) WHERE a.aid = {id}", id=self.aid);
        let rawqry = pgconn.query(&qrystr, &[]);
        
        // println!("Running query:\n{}", qrystr);
        if let Ok(aqry) = rawqry {
            println!("Querying articles: found {} rows", aqry.len());
            if !aqry.is_empty() && aqry.len() == 1 {
                let row = aqry.get(0); // get first row
                let display: Option<String> = row.get(7);
                let username: String = if let Some(disp) = display { disp } else { row.get(8) };
                let image: String = row.get_opt(9).unwrap_or(Ok(String::new())).unwrap_or(String::new());
                Some( Article {
                    aid: row.get(0),
                    title: row.get(1), // todo: call sanitize title here
                    posted: row.get(2),
                    body: row.get(3), // Todo: call sanitize body here
                    tags: row.get_opt(4).unwrap_or(Ok(Vec::<String>::new())).unwrap_or(Vec::<String>::new()).into_iter().map(|s| s.trim_matches('\'').trim().to_string()).filter(|s| s.as_str() != "").collect(),
                    description: row.get_opt(5).unwrap_or(Ok(String::new())).unwrap_or(String::new()),
                    userid: row.get(6),
                    username: titlecase( &sanitization::sanitize(&username) ),
                    markdown: row.get_opt(10).unwrap_or(Ok(String::new())).unwrap_or(String::new()),
                    image,
                    modified: row.get(11),
                })
            } else { None }
        } else { None }
    }
}


impl Search {
    pub fn search() -> Vec<Article> {
        unimplemented!()
    }
}

pub fn get_len<T>(input: &Option<Vec<T>>) -> usize {
    if let &Some(ref inner) = input {
        inner.len()
    } else {
        0
    }
}

pub fn slash_quotes(text: &str) -> String {
    // text.replace("\\", "").replace("'", "\\'").replace("\"", "\\\"")
    text.replace("'", "''")
}

impl Article {
    pub fn short_clone(&self) -> Article {
        Article {
            aid: self.aid,
            title: self.title.to_owned(),
            posted: self.posted.clone(),
            modified: self.modified.clone(),
            userid: self.userid,
            username: self.username.to_owned(),
            body: if self.description != "" {
                self.description.to_owned()
            } else {
                self.body[..DESC_LIMIT].to_owned()
            },
            tags: self.tags.clone(),
            description: self.description.to_owned(),
            markdown: self.markdown.to_owned(),
            image: self.image.to_owned()
        }
    }
    pub fn to_display(&self) -> ArticleDisplay {
        ArticleDisplay {
            aid: self.aid.clone(),
            title: self.title.clone(),
            posted_machine: self.posted.format("%Y-%m-%dT%H:%M:%S").to_string(),
            posted_human: self.posted.format("%Y-%m-%d @ %I:%M%P").to_string(),
            body: self.body.clone().replace("{{base_url}}", BLOG_URL),
            tags: self.tags.clone(),
            description: self.description.clone(),
            userid: self.userid,
            username: self.username.clone(),
            markdown: self.markdown.clone(),
            image: self.image.clone(),
            modified_machine: if &self.posted != &self.modified { self.modified.format("%Y-%m-%dT%H:%M:%S").to_string() } else { String::new() },
            modified_human: if &self.posted != &self.modified { self.modified.format("%Y-%m-%d @ %I:%M%P").to_string() } else { String::new() },
        }
    }
    pub fn split_tags(string: String) -> Vec<String> {
        // Todo: call sanitize tags before splitting:
        let tags: Vec<String> = string.split(',')
            .map( |s| sanitize_tag(s.trim()) )
            .filter(|s| s.as_str() != "" && s.as_str() != " ")
            .collect();
        tags
    }
    pub fn retrieve(aid: u32) -> Option<Article> {
        let pgconn = establish_connection();
        let rawqry = pgconn.query(&format!("SELECT a.aid, a.title, a.posted, a.body, a.tag, a.description, u.userid, u.display, u.username, a.image, a.markdown, a.modified FROM articles a JOIN users u ON (a.author = u.userid) WHERE aid = {id}", id=aid), &[]);
        if let Ok(aqry) = rawqry {
            // println!("Querying articles: found {} rows", aqry.len());
            if !aqry.is_empty() && aqry.len() == 1 {
                let row = aqry.get(0); // get first row
                
                let display: Option<String> = row.get(7);
                let username: String = if let Some(disp) = display { disp } else { row.get(8) };
                let image: String = row.get_opt(9).unwrap_or(Ok(String::new())).unwrap_or(String::new());
                
                Some( Article {
                    aid: row.get(0),
                    title: row.get(1), // todo: call sanitize title here
                    posted: row.get(2),
                    body: row.get(3), // Todo: call sanitize body here
                    tags: row.get_opt(4).unwrap_or(Ok(Vec::<String>::new())).unwrap_or(Vec::<String>::new()).into_iter().map(|s| s.trim_matches('\'').trim().to_string()).filter(|s| s.as_str() != "").collect(),
                    description: row.get_opt(5).unwrap_or(Ok(String::new())).unwrap_or(String::new()),
                    userid: row.get(6),
                    username: titlecase( &sanitization::sanitize(&username) ),
                    markdown: row.get_opt(10).unwrap_or(Ok(String::new())).unwrap_or(String::new()),
                    image,
                    modified: row.get(11),
                })
            } else { None }
        } else { None }
    }
    
    
    // Updates an article.  Use the newer, preferred method update()
    // which takes a reference to a connection instead of an owned connection.
    // The article being saved must exist already.  The method looks for
    // an article with the same `aid` as the `self`'s `aid`.
    pub fn save(&self, conn: DbConn) -> Result<String, String> {
        let vtags: Vec<String> = self.tags.clone();
        let tagstr = format!( 
            "{{{}}}", vtags
            .iter()
            .map(|s| s.trim().to_lowercase())
            .filter(|s| s.as_str() != "")
            .map(|s| format!("\"{}\"", s))
            .collect::<Vec<_>>()
            .join(",")
        );
        let now = Local::now().naive_local();
        let qrystr = format!("
            UPDATE articles 
                SET title = '{title}',
                    body = '{body}',
                    tag = '{tag}',
                    description = '{desc}',
                    markdown = '{src}',
                    image = '{img}',
                    modified = '{modified}'
                WHERE aid = {aid}
            ", 
            title=&self.title, 
            body=&self.body,
            tag=tagstr,
            desc=&self.description,
            src=&self.markdown,
            img=&self.image,
            aid=self.aid,
            modified=&now
        );
        
        // println!("Generated update query:\n{}", qrystr);
        
        if let Ok(num) = conn.execute(&qrystr, &[]) {
            if num == 1 {
                Ok(format!("Article {} successfully updated", self.aid))
            } else if num > 1 {
                println!("Update query updated too many rows.");
                Err("Multiple rows updated".to_string())
            } else {
                println!("Update query updated no rows.");
                Err(String::new())
            }
        } else {
            println!("Update query failed.");
            Err(String::new())
        }
        
    } 
    
    // Same as the save() method except it takes a reference to a connection.
    // This is the preferred method for updating/saving an existing article.
    pub fn update(&self, conn: &DbConn) -> Result<String, String> {
        let vtags: Vec<String> = self.tags.clone();
        let tagstr = format!( 
            "{{{}}}", vtags
            .iter()
            .map(|s| s.trim().to_lowercase())
            .filter(|s| s.as_str() != "")
            .map(|s| format!("\"{}\"", s))
            .collect::<Vec<_>>()
            .join(",")
        );
        let now = Local::now().naive_local();
        let qrystr = format!("
            UPDATE articles 
                SET title = '{title}',
                    body = '{body}',
                    tag = '{tag}',
                    description = '{desc}',
                    markdown = '{src}',
                    image = '{img}',
                    modified = '{modified}'
                WHERE aid = {aid}
            ", 
            title=&self.title, 
            body=&self.body,
            tag=tagstr,
            desc=&self.description,
            src=&self.markdown,
            img=&self.image,
            aid=self.aid,
            modified=&now
        );
        
        // println!("Generated update query:\n{}", qrystr);
        
        if let Ok(num) = conn.execute(&qrystr, &[]) {
            if num == 1 {
                Ok(format!("Article {} successfully updated", self.aid))
            } else if num > 1 {
                println!("Update query updated too many rows.");
                Err("Multiple rows updated".to_string())
            } else {
                println!("Update query updated no rows.");
                Err(String::new())
            }
        } else {
            println!("Update query failed.");
            Err(String::new())
        }
        
    } 
    
    pub fn info(&self) -> String {
        format!("Aid: {aid}, Title: {title}, Posted on: {posted}, Description:<br>\n{desc}<br>\nBody:<br>\n{body}<br>\ntags: {tags:#?}", aid=self.aid, title=self.title, posted=self.posted, body=self.body, tags=self.tags, desc=self.description)
    }
    
    
    /// Description: Some(50) displays 50 characters of body text
    ///              Some(-1) displays the description field as the body
    ///              None displays all of the body text
    pub fn retrieve_all(pgconn: DbConn, limit: u32, description: Option<i32>, min_date: Option<NaiveDate>, max_date: Option<NaiveDate>, tag: Option<Vec<String>>, search: Option<Vec<String>>) -> Vec<Article> {
        let mut show_desc = false;
        let mut qrystr: String = if let Some(summary) = description {
            if summary < 1 {
                show_desc = true;
                format!("SELECT a.aid, a.title, a.posted, LEFT(a.body, {}) as body, a.tag, a.description, u.userid, u.display, u.username, a.image, a.markdown, a.modified FROM articles a JOIN users u ON(a.author = u.userid)", DESC_LIMIT)
            } else {
                format!("SELECT a.aid, a.title, a.posted, LEFT(a.body, {}) AS body, a.tag, a.description, u.userid, u.display, u.username, a.image, a.markdown, a.modified FROM articles a JOIN users u ON(a.author = u.userid)", summary)
            }
        } else {
            String::from("SELECT a.aid, a.title, a.posted, a.body, a.tag, a.description, u.userid, u.display, u.username, a.image, a.markdown, a.modified FROM articles a JOIN users u ON(a.author = u.userid)")
        };
        if min_date.is_some() || max_date.is_some() || (tag.is_some() && get_len(&tag) != 0) || (search.is_some() && get_len(&search) != 0) {
            qrystr.push_str(" WHERE");
            let mut where_str = String::from("");
            if let Some(date_min) = min_date {
                where_str.push_str( &format!(" posted >= '{}'", date_min.format("%Y-%m-%d %H:%M:%S")) );
            }
            if let Some(date_max) = max_date {
                if &where_str != "" { where_str.push_str(" AND "); }
                where_str.push_str( &format!(" posted <= '{}'", date_max.format("%Y-%m-%d %H:%M:%S")) );
                
            }
            if let Some(v) = tag {
                if &where_str != "" { where_str.push_str(" AND "); }
                let mut tag_str = String::new();
                let mut first: bool = true;
                for t in v {
                    if first { first = false; } else { tag_str.push_str(" AND "); }
                    tag_str.push_str( &format!(" '{}' = ANY(tag)", t) );
                }
                if &tag_str != "" { where_str.push_str(&tag_str); }
            }
            if let Some(strings) = search {
                if &where_str != "" { where_str.push_str(" AND "); }
                let mut search_str = String::new();
                let mut first: bool = true;
                for string in strings {
                    if first { first = false; } else { search_str.push_str(" AND ") }
                    search_str.push_str( &format!(" (title LIKE '%{s}%' OR body LIKE '%{s}%')", s=string) );
                }
                if &search_str != "" { where_str.push_str(&search_str); }
            }
            qrystr.push_str(&where_str);
        }
        qrystr.push_str(" ORDER BY posted DESC");
        if limit != 0 { qrystr.push_str(&format!(" LIMIT {}", limit)); }
        // println!("Query: {}", qrystr);
        let qryrst = pgconn.query(&qrystr, &[]);
        if let Ok(result) = qryrst {
            let mut articles: Vec<Article> = Vec::new();
            
            for row in &result {
                let display: Option<String> = row.get(7);
                let username: String = if let Some(disp) = display { disp } else { row.get(8) };
                let image: String = row.get_opt(9).unwrap_or(Ok(String::new())).unwrap_or(String::new());
                
                let a = Article {
                    aid: row.get(0),
                    title: row.get(1),
                    posted: row.get(2),
                    body: if show_desc {  // show the truncated body if there is no description when show_desc is true
                            let d = row.get_opt(5).unwrap_or(Ok(String::new())).unwrap_or(String::new());
                            if &d == "" { row.get(3) }
                            else { d }
                        } else { row.get(3) },
                    tags: row.get_opt(4).unwrap_or(Ok(Vec::<String>::new())).unwrap_or(Vec::<String>::new()).into_iter().map(|s| s.trim_matches('\'').trim().to_string()).filter(|s| s.as_str() != "").collect(),
                    description: if show_desc { 
                            String::new() 
                        } else { 
                            row.get_opt(5).unwrap_or(Ok(String::new())).unwrap_or(String::new()) 
                        },
                    userid: row.get(6),
                    username: titlecase( &sanitization::sanitize(&username) ),
                    markdown: row.get_opt(10).unwrap_or(Ok(String::new())).unwrap_or(String::new()),
                    image,
                    modified: row.get(11)
                    
                };
                articles.push(a);
            }
            // println!("Found {} articles with the specified query.", articles.len());
            articles
        } else {
            println!("Query failed.");
            Vec::<Article>::new()
        }
    }
}

impl PartialEq for Article {
    fn eq(&self, other: &Article) -> bool {
        self.aid == other.aid
    }
}

impl Eq for Article {}

impl ShallowCopy for Article {
    unsafe fn shallow_copy(&mut self) -> Self {
        self.clone()
    }
}




impl ArticleForm {
    pub fn new(title: String, body: String, tags: String, description: String, markdown: String, image: String) -> ArticleForm {
        ArticleForm {
            title,
            body,
            markdown,
            tags,
            description,
            image,
        }
    }
    pub fn is_valid(&self) -> bool {
        &self.title != ""
        && ( &self.markdown != "" || &self.body != "" )
    }
    pub fn save(self, conn: &DbConn, userid: u32, username: &str) -> Result<Article, String> {
        let now = Local::now().naive_local();
        
        // take blah, blah2, blah3 and convert into {'blah', 'blah2', 'blah3'}
        let tagstr = format!( 
            "{{{}}}", self.tags.clone()
            .split(',')
            .map(|s| s.trim().to_lowercase())
            .filter(|s| s.as_str() != "")
            .map(|s| format!("\"{}\"", s))
            .collect::<Vec<_>>()
            .join(",")
        );
            
        //  can return both id and posted date
        let qrystr = format!("INSERT INTO articles (title, posted, body, tag, description, author, markdown, image, modified) VALUES ('{title}', '{posted}', '{body}', '{tags}', '{desc}', {author}, '{md}', '{img}', '{posted}') RETURNING aid",
            title=&self.title, posted=&now, body=&self.body, tags=tagstr, desc=&self.description, author=userid, md=&self.markdown, img=&self.image);
        // println!("Insert query: {}", qrystr);
        let result = conn.query(&qrystr, &[]);
        match result {
            Err(err) => Err(format!("Could not insert article. Error: {}", err)),
            Ok(qry)  => {
                if !qry.is_empty() && qry.len() == 1 {
                    let row = qry.get(0);
                    Ok( Article {
                        aid: row.get(0),
                        title: self.title,
                        posted: now.clone(),
                        body: self.body,
                        markdown: self.markdown,
                        tags: Article::split_tags(self.tags),
                        description: self.description,
                        userid,
                        username: titlecase( &sanitization::sanitize(username) ), 
                        image: self.image,
                        modified: now,
                    })
                } else if qry.is_empty() {
                    Err("Error inserting article, result is empty.".to_string())
                } else if qry.len() > 1 {
                    Err(format!("Error inserting article, returned {} rows.", qry.len()))
                } else {
                    Err("Unknown error inserting article.".to_string())
                }
            },
        }
    }
}


impl<'f> FromForm<'f> for ArticleId {
    type Error = &'static str;
    fn from_form(form_items: &mut FormItems<'f>, _strict: bool) -> Result<Self, Self::Error> {
        let mut aid: u32 = 0;
        for (field, value) in form_items {
            if field.as_str() == "aid" {
                aid = value.to_string().parse::<u32>().unwrap_or(0u32);
                return Ok( ArticleId { aid } );
            }
        }
        Ok( ArticleId { aid} )
    }
}


impl<'f> FromForm<'f> for Search {
    type Error = &'static str;
    fn from_form(form_items: &mut FormItems<'f>, _strict: bool) -> Result<Self, Self::Error> {
        let mut tag: String = String::new();
        let mut limit: Option<u16> = None;
        let mut o: Option<String> = None;
        let mut p: Option<String> = None;
        let mut q: Option<String> = None;
        let mut min: Option<NaiveDateTimeWrapper> = None;
        let mut max: Option<NaiveDateTimeWrapper> = None;
        for (field, value) in form_items {
            match field.as_str() {
                "limit" => { limit = Some(value.parse::<u16>().unwrap_or(0)) },
                "o" => { o = Some(encode_attribute(&value.url_decode().unwrap_or(String::new())) ) },
                "p" => { p = Some(encode_attribute(&value.url_decode().unwrap_or(String::new())) ) },
                "q" => { q = Some(encode_attribute(&value.url_decode().unwrap_or(String::new())) ) },
                "min" => { min = if let Ok(m) = value.as_str().parse::<NaiveDateTime>() { Some(NaiveDateTimeWrapper(m)) } else { None } },
                "max" => { max = if let Ok(m) = value.as_str().parse::<NaiveDateTime>() { Some(NaiveDateTimeWrapper(m)) } else { None } },
                _ => {},
            }
        }
        Ok(Search {
            limit,
            o,
            p,
            q,
            min,
            max,
        })
    }
}



impl<'f> FromForm<'f> for Tag {
    type Error = &'static str;
    fn from_form(form_items: &mut FormItems<'f>, _strict: bool) -> Result<Self, Self::Error> {
        let mut tag: String = String::new();
        for (field, value) in form_items {
            if field.as_str() == "tag" {
                tag = value.url_decode().expect("URL Decode fail."); 
            }
        }
        if &tag == ""  {
            Err("No tag specified")
        } else {
            Ok( Tag{ tag } )
        }
    }
}

impl<'f> FromForm<'f> for ArticleForm {
    type Error = &'static str;
    
    fn from_form(form_items: &mut FormItems<'f>, _strict: bool) -> Result<Self, Self::Error> {
        
        // Author should be blank string here, when saving the author can be identified from cookies
        // this prevents user from altering the userid in submitted form data when using a hidden field to save the userid
        
        let mut title: String = String::new();
        let mut body: String = String::new();
        let mut tags: String = String::new();
        let mut description: String = String::new();
        let mut markdown: String = String::new();
        let mut image: String = String::new();
        
        for (field, value) in form_items {
            match field.as_str() {
                "title" => { title = escape_sql_pg(sanitize_title(value.url_decode().unwrap_or( String::new() ))) },
                "body" => { body = escape_sql_pg(value.url_decode().unwrap_or( String::new() )) },
                "markdown" => { markdown = escape_sql_pg(value.url_decode().unwrap_or( String::new() )) },
                "tags" => { tags = escape_sql_pg(sanitize_tags(value.url_decode().unwrap_or( String::new() ))) },
                "description" => { description = escape_sql_pg(value.url_decode().unwrap_or( String::new() )) },
                "image" => { image = escape_sql_pg(value.url_decode().unwrap_or( String::new() )) },
                _ => {},
            }
        }
        if title.len()+1 > MAX_CREATE_TITLE {
            title = title[..MAX_CREATE_TITLE].to_string();
        }
        if tags.len()+1 > MAX_CREATE_TAGS {
            tags = tags[..MAX_CREATE_TAGS].to_string();
        }
        if description.len()+1 > MAX_CREATE_DESCRIPTION {
            description = description[..MAX_CREATE_DESCRIPTION].to_string();
        }
        // if title == "" || body == "" {
            // Err("Missing a required field.")
        // } else {
            Ok( ArticleForm::new(title, body, tags, description, markdown, image) )
        // }
    }
}


impl<'f> FromForm<'f> for ArticleWrapper {
    type Error = &'static str;
    
    fn from_form(form_items: &mut FormItems<'f>, _strict: bool) -> Result<Self, Self::Error> {
        
        // Author should be blank string here, when saving the author can be identified from cookies
        // this prevents user from altering the userid in submitted form data when using a hidden field to save the userid
        
        let mut aid: u32 = 0;
        let mut posted: NaiveDateTimeWrapper = NaiveDateTimeWrapper(now());
        let mut userid: u32 = 0;
        let mut username: String = String::new();
        let mut title: String = String::new();
        let mut body: String = String::new();
        let mut tags: String = String::new();
        let mut description: String = String::new();
        let mut markdown: String = String::new();
        let mut image: String = String::new();
        // modified should not be determined by a form field
        let mut modified: NaiveDateTimeWrapper = NaiveDateTimeWrapper(now());
        
        for (field, value) in form_items {
            match field.as_str() {
                "aid" => { aid = value.parse::<u32>().unwrap_or(0u32) },
                "posted" => { posted = NaiveDateTimeWrapper::from_form_value(value).unwrap_or( NaiveDateTimeWrapper(now()) ) },
                "userid" => { userid = value.parse::<u32>().unwrap_or(0u32) },
                "username" => { username = escape_sql_pg(value.url_decode().unwrap_or( String::new() )) },
                "title" => { title = escape_sql_pg(sanitize_title(value.url_decode().unwrap_or( String::new() ))) },
                "body" => { body = escape_sql_pg(value.url_decode().unwrap_or( String::new() )) },
                "markdown" => { markdown = escape_sql_pg(value.url_decode().unwrap_or( String::new() )) },
                "tags" => { tags = escape_sql_pg(sanitize_tags(value.url_decode().unwrap_or( String::new() ))) },
                "description" => { description = escape_sql_pg(value.url_decode().unwrap_or( String::new() )) },
                "image" => { image = escape_sql_pg(value.url_decode().unwrap_or( String::new() )) },
                _ => {},
            }
        }
        // if title == "" || body == "" {
            // Err("Missing a required field.")
        // } else {
            Ok( ArticleWrapper {
                aid,
                title,
                posted,
                userid,
                username,
                body,
                tags,
                description,
                markdown,
                image,
                modified,
            } )
        // }
    }
}



impl<'v> FromFormValue<'v> for NaiveDateTimeWrapper {
    type Error = ();

    fn from_form_value(form_value: &'v RawStr) -> Result<NaiveDateTimeWrapper, ()> {
        let val = form_value.url_decode().unwrap_or(String::new());
        match val.parse::<NaiveDateTime>() {
            Ok(date) => Ok(NaiveDateTimeWrapper(date)),
            Err(e) => {
                println!("\n\nError processing NaiveDateTimeWrapper\nTried to process: {}\nOriginal Input: {}\nReturned Error: {}\n", val, form_value, e);
                Err(())
            },
        }
    }
}



impl<'r> FromParam<'r> for ArticleId {
    type Error = &'r str;

    fn from_param(param: &'r RawStr) -> Result<Self, Self::Error> {
        // https://api.rocket.rs/rocket/request/trait.FromParam.html
        
        match param.parse::<u32>() {
            Ok(i) => Ok(ArticleId { aid: i }),
            Err(e) => Err("Could not convert id.")
        }
    }
}




impl<'r> FromParam<'r> for Search {
    type Error = &'r RawStr;

    fn from_param(param: &'r RawStr) -> Result<Self, Self::Error> {
        use chrono::format::ParseError;
        
        let decoded_opt = param.url_decode();
        if let Ok(decoded) = decoded_opt {
            
            let parts: Vec<Vec<_>> = decoded
                .split("&")
                .map( |p|
                    p.split("=").collect()
                ).collect();
            
            let mut q: String = String::new();
            let mut min: Option<NaiveDateTimeWrapper> = None;
            let mut max: Option<NaiveDateTimeWrapper> = None;
            
            let mut min_rst: Option<Result<NaiveDateTime, ParseError>> = None;
            let mut max_rst: Option<Result<NaiveDateTime, ParseError>> = None;
            
            for v in &parts {
                if v.len() == 2 {
                    match v[0] {
                        "q" => { q = sanitization::sanitize_text(v[1]); },
                        "min" => { min_rst = Some(NaiveDateTime::parse_from_str(&v[1], "%Y-%m-%d %H:%M:%S")); },
                        "max" => { max_rst = Some(NaiveDateTime::parse_from_str(&v[1], "%Y-%m-%d %H:%M:%S")); },
                        _ => {},
                    }
                }
            }
            
            if let Some(Ok(min_date)) = min_rst {
                let min = Some( NaiveDateTimeWrapper(min_date) );
            }
            if let Some(Ok(max_date)) = max_rst {
                let max = Some( NaiveDateTimeWrapper(max_date) );
            }
            
            Ok(Search {
                limit: None,
                o: None,
                p: None,
                q: Some(q),
                min,
                max
            })
        } else {
            Err(param)
        }
    }
}



