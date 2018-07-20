
use ::rocket::request::{self, FromRequest, FromForm, FormItems};
use rocket::{Request, State, Outcome};
use ::rocket::config::{Config, Environment};
use rocket::http::Status;

use regex::Regex;
use chrono::prelude::*;
use chrono::{NaiveDate, NaiveDateTime};

use titlecase::titlecase;


use r2d2;
use r2d2_postgres;
use r2d2_postgres::{PostgresConnectionManager, TlsMode};
use postgres::Connection;
use postgres;
use postgres::params::{ConnectParams, Host};

use std::ops::Deref;
use std;
use std::env;


use blog::*;
use super::{DATABASE_URL, DESC_LIMIT};


// https://github.com/sfackler/rust-postgres/issues/128
// let stmt = try!(conn.prepare("INSERT INTO foo (bar) VALUES ('baz') RETURNING id"));
// let id: i32 = try!(stmt.query(&[])).iter().next().unwrap().get(0);




// https://sfackler.github.io/r2d2-postgres/doc/v0.9.2/r2d2_postgres/struct.PostgresConnectionManager.html
// https://medium.com/@aergonaut/writing-a-github-webhook-with-rust-part-1-rocket-4426dd06d45d
// https://github.com/aergonaut/railgun/blob/master/src/railgun/db.rs

/// Type alias for the r2d2 connection pool. Use this as a State<T> parameter
/// in handlers that need a database connection.
// pub type ConnectionPool = r2d2::Pool<r2d2_diesel::ConnectionManager<diesel::pg::PgConnection>>;
type Pool = r2d2::Pool<PostgresConnectionManager>;



/// Creates the database connection pool
pub fn init_pg_pool() -> Pool {
    let config = r2d2::Config::default();
    let manager = PostgresConnectionManager::new(DATABASE_URL, TlsMode::None).expect("Could not connect to database using specified connection string.");
    
    r2d2::Pool::new(config, manager).expect("Could not create database pool")
}

pub fn init_pg_conn() -> Connection {
    Connection::connect(DATABASE_URL, postgres::TlsMode::None).unwrap()
}

pub struct DbConn(
    pub r2d2::PooledConnection<PostgresConnectionManager>
);

impl DbConn {
    /// If called like: conn.articles("") it will return all articles.  The description of the article is used if it exists otherwise a truncated body is returned; to return articles will their full body contents use `conn.articles_full("")`.
    pub fn articles(&self, qrystr: &str) -> Option<Vec<Article>> {
        
        let qrystring: String;
        let qrystr = if qrystr == "" {
            qrystring = format!("SELECT a.aid, a.title, a.posted, description({}, a.body, a.description), a.tag, a.description, u.userid, u.display, u.username, a.image, a.markdown, a.modified FROM articles a JOIN users u ON (a.author = u.userid) ORDER BY a.posted DESC", DESC_LIMIT);
            &qrystring
        } else {
            qrystr
        };
        
        // let qryrst: Result<_, _> = if qrystr != "" {
            // self.query(qrystr, &[])
        // } else {
            // self.query(&format!("SELECT a.aid, a.title, a.posted, description({}, a.body, a.description), a.tag, a.description, u.userid, u.display, u.username, a.image, a.markdown, a.modified FROM articles a JOIN users u ON (a.author = u.userid) ODER BY a.posted DESC", DESC_LIMIT), &[])
        // };
        let qryrst = self.query(&qrystr, &[]);
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
                    body: row.get(3),
                    tags: row.get_opt(4).unwrap_or(Ok(Vec::<String>::new())).unwrap_or(Vec::<String>::new()).into_iter().map(|s| s.trim_matches('\'').trim().to_string()).filter(|s| s.as_str() != "").collect(),
                    description: row.get_opt(5).unwrap_or(Ok(String::new())).unwrap_or(String::new()),
                    userid: row.get(6),
                    username: titlecase( &username ),
                    markdown: row.get_opt(10).unwrap_or(Ok(String::new())).unwrap_or(String::new()),
                    image,
                    modified: row.get(11),
                };
                articles.push(a);
            }
            Some(articles)
        } else {
            println!("Attempted to retrieve articles using the following query: {}", qrystr);
            None
        }
    }
    /// Runs a query returning articles from the database.  If the text passed in is equal to "" then the default 
    /// query is to return all articles with full body content.
    pub fn articles_full(&self, qrystr: &str) -> Option<Vec<Article>> {
        let qry = if qrystr != "" { qrystr } else { "SELECT a.aid, a.title, a.posted, a.body, a.tag, a.description, u.userid, u.display, u.username, a.image, a.markdown, a.modified  FROM articles a JOIN users u ON (a.author = u.userid) ORDER BY a.posted DESC" };
        self.articles(qry)
    }
}

/// Attempts to retrieve a single connection from the managed database pool. If
/// no pool is currently managed, fails with an `InternalServerError` status. If
/// no connections are available, fails with a `ServiceUnavailable` status.
impl<'a, 'r> FromRequest<'a, 'r> for DbConn {
    type Error = ();

    fn from_request(request: &'a Request<'r>) -> request::Outcome<DbConn, ()> {
        let pool = request.guard::<State<Pool>>()?;
        match pool.get() {
            Ok(conn) => Outcome::Success(DbConn(conn)),
            Err(_) => Outcome::Failure((Status::ServiceUnavailable, ()))
        }
    }
}

// For the convenience of using an &DbConn as an &SqliteConnection.
impl Deref for DbConn {
    // type Target = SqliteConnection;
    type Target = Connection;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}


pub fn establish_connection() -> Connection {
    Connection::connect(DATABASE_URL, postgres::TlsMode::None).unwrap()
}

// Commented out because the DotEnv crate isn't required anywhere else
// and these functions are not used.  They are left here because they
// could be useful at some point, someday but not immediately.


