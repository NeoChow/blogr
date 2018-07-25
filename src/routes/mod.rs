
use std::{thread, time};
use std::time::Instant;
use std::time::Duration;
use std::{env, str, io};
use std::io::{Cursor, Read};
use std::path::{Path, PathBuf};
use std::io::prelude::*;
use std::fs::{self, File};

use rocket_contrib::Template;
use rocket::response::{content, NamedFile, Redirect, Flash};
use rocket::{Request, Data, Outcome};
use rocket::request::{FlashMessage, Form, FromForm};
use rocket::data::FromData;
use rocket::response::content::Html;
use rocket::State;
use rocket::http::{Cookie, Cookies, RawStr};
use regex::Regex;
use titlecase::titlecase;

use chrono::prelude::*;
use chrono::{NaiveDate, NaiveDateTime};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Mutex, Arc, RwLock};

use rocket::http::hyper::header::{Headers, ContentDisposition, DispositionType, DispositionParam, Charset};



use super::*;
use cache::*;
use content::{destruct_cache, destruct_context};
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


use comrak::{markdown_to_html, ComrakOptions};


fn err_file_name(name: &str) -> PathBuf {
    if let Ok(mut dir) = env::current_exe() {
        dir.pop();
        // println!("Climbing directory tree into: {}", &dir.display());
        dir.pop();
        // println!("Loading into directory: {}", &dir.display());
        if cfg!(target_os = "windows") {
            dir.set_file_name(&format!("logs\\{}", name));
        } else {
            dir.set_file_name(&format!("logs/{}", name));
        }
        // println!("Load file is: {}", &dir.display());
        dir
    } else {
        PathBuf::from(name)
    }
}

#[cfg(PRODUCTION)]
fn debug_timer(_: &GenTimer) { }

#[cfg(not(PRODUCTION))]
fn debug_timer(start: &GenTimer) {
    let end = start.0.elapsed();
    println!("Served in {}.{:09} seconds", end.as_secs(), end.subsec_nanos());
}

#[inline]
fn fmsg(flash: Option<FlashMessage>) -> Option<String> {
    if let Some(flashmsg) = flash {
        Some(alert_info( flashmsg.msg() ))
    } else {
        None
    }
}


pub mod tagcloud {
    use super::*;

    // replaces route pages::hbs_tags_all
    #[get("/all_tags")]
    pub fn cache_tagcloud(start: GenTimer, 
                        multi_aids: State<TagAidsLock>,
                        conn: DbConn,
                        admin: Option<AdministratorCookie>,
                        user: Option<UserCookie>,
                        encoding: AcceptCompression,
                        uhits: UniqueHits
                    ) -> Express 
    {
        let express: Express = cache::pages::tags::serve(&conn,
                                                        &multi_aids,
                                                        admin,
                                                        user,
                                                        Some(uhits),
                                                        Some(start.clone()),
                                                        Some(encoding),
                                                        None
                                                        );
        debug_timer(&start);
        express.compress( encoding )
    }
}

pub mod tag {
    use super::*;

    #[get("/tag?<tag>")]
    pub fn cache_tag_redirect(tag: Tag) -> Redirect {
        Redirect::to(&format!("/tag/{}", tag.tag))
    }

    #[get("/tag/<tag>")]
    pub fn cache_tag(start: GenTimer,
                            tag: String,
                            multi_aids: State<TagAidsLock>, 
                            article_state: State<ArticleCacheLock>, 
                            pagination: Page<Pagination>,
                            conn: DbConn, 
                            admin: Option<AdministratorCookie>, 
                            user: Option<UserCookie>, 
                            encoding: AcceptCompression, 
                            uhits: UniqueHits
                        ) -> Express 
    {
        
        let express: Express = cache::pages::tag::serve(&tag, 
                                                        &pagination, 
                                                        &*multi_aids, 
                                                        &*article_state, 
                                                        &conn, 
                                                        admin, 
                                                        user, 
                                                        Some(uhits), 
                                                        Some(start.clone()), 
                                                        Some(encoding), 
                                                        None
                                                    );
        debug_timer(&start);
        express.compress( encoding )
    }
}

pub mod article {
    use super::*;

    #[get("/article/<aid>/<title>")]
    pub fn cache_article_title(start: GenTimer,
                               aid: ArticleId, 
                               title: Option<&RawStr>,
                               article_lock: State<ArticleCacheLock>,
                               conn: DbConn,
                               admin: Option<AdministratorCookie>,
                               user: Option<UserCookie>,
                               encoding: AcceptCompression,
                               uhits: UniqueHits
                              ) -> Express 
    {
        routes::article::cache_article_view(start, aid, article_lock, conn, admin, user, encoding, uhits)
    }

    #[get("/article/<aid>")]
    pub fn cache_article_id(start: GenTimer,
                            aid: ArticleId, 
                            article_lock: State<ArticleCacheLock>,
                            conn: DbConn,
                            admin: Option<AdministratorCookie>,
                            user: Option<UserCookie>,
                            encoding: AcceptCompression,
                            uhits: UniqueHits
                           ) -> Express 
    {
        routes::article::cache_article_view(start, aid, article_lock, conn, admin, user, encoding, uhits)
    }

    #[get("/article?<aid>")]
    pub fn cache_article_view(start: GenTimer, 
                              aid: ArticleId,
                              article_state: State<ArticleCacheLock>, 
                              conn: DbConn, 
                              admin: Option<AdministratorCookie>, 
                              user: Option<UserCookie>, 
                              encoding: AcceptCompression, 
                              uhits: UniqueHits
                          ) -> Express 
    {
        let express: Express = cache::pages::article::serve(aid.aid, 
                                                            article_state, 
                                                            &conn, 
                                                            admin, 
                                                            user, 
                                                            start.clone(),
                                                            uhits,
                                                            encoding, 
                                                            None
                                                        );
        debug_timer(&start);
        express.compress( encoding )
    }

    #[get("/article")]
    pub fn hbs_article_not_found(start: GenTimer, conn: DbConn, admin: Option<AdministratorCookie>, user: Option<UserCookie>, encoding: AcceptCompression, uhits: UniqueHits) -> Express {
        let output: Template = hbs_template(TemplateBody::General(alert_danger("Article not found")), None, Some("Article not found".to_string()), String::from("/article"), admin, user, None, Some(start.0));
        let end = start.0.elapsed();
        // println!("Served in {}.{:09} seconds", end.as_secs(), end.subsec_nanos());
        let express: Express = output.into();
        express.compress(encoding)
    }
}

pub mod rss {
    use super::*;
    
    #[get("/author_feed/<author>")]
    pub fn rss_author_filter(start: GenTimer,
                      author: u32,
                      text_lock: State<TextCacheLock>, 
                      encoding: AcceptCompression, 
                      uhits: UniqueHits
                     ) -> Express
    {
        unimplemented!();
    }
    
    
    #[get("/rss-userid/<userid>")]
    pub fn rss_userid_filter(start: GenTimer,
                             userid: u32,
                             text_lock: State<TextCacheLock>,
                             encoding: AcceptCompression,
                             uhits: UniqueHits
                            ) -> Express
    {
        // basically just do: text_lock.retrieve_text(&format!("rss-author/{}", userid))
        // unimplemented!()
        cache::pages::rss::serve_filter(format!("rss-author/{}", userid), &text_lock, Some(uhits), Some(start), Some(encoding))
    }
    
    #[get("/rss-username/<username>")]
    pub fn rss_username_filter(start: GenTimer,
                               conn: DbConn,
                               username: String,
                               text_lock: State<TextCacheLock>,
                               encoding: AcceptCompression,
                               uhits: UniqueHits
                              ) -> Express
    {
        // do a database query for a user with that name, return the userid
        // if there is a valid userid call rss_userid_filter()
        let username = ::sanitize::escape_sql_pg(::sanitize::medium_sanitize(username).to_lowercase());
        // let username = ::sanitize::escape_sql_pg(::sanitize::sanitize_tag(&username).to_lowercase());
        let qrytext = format!("SELECT userid FROM users WHERE lower(username) = '{user}' OR lower(display) = '{user}'", user=username);
        let qry = conn.query(&qrytext, &[]);
        if let Ok(result) = qry {
            if !result.is_empty() && result.len() == 1 {
               let row = result.get(0);
               let userid = row.get(0);
               rss_userid_filter(start, userid, text_lock, encoding, uhits)
                
            } else {
            //    let express: Express = "Username not found".to_owned().into();
               let express: Express = format!("Username '{}' not found", username).into();
               express
            }
        } else if let Err(err) = qry {
               let express: Express = format!("Could not execute database query: '{}'\n{}", err, qrytext).into();
               express
        } else {
               let express: Express = "Could not reach database.".to_owned().into();
               express
        }
        
        
        // unimplemented!()
    }
    
    
    #[get("/rss-tag/<tag>")]
    pub fn rss_tag_filter(start: GenTimer,
                          tag: String,
                          text_lock: State<TextCacheLock>, 
                          encoding: AcceptCompression, 
                          uhits: UniqueHits
                         ) -> Express
    {
        // Basically just do: text_lock.retrieve_text(&format!("rss-tag/{}", tag))
        let tag = sanitize::sanitize_tags(tag);
        cache::pages::rss::serve_filter(format!("rss-tag/{}", tag), &text_lock, Some(uhits), Some(start), Some(encoding))
        // unimplemented!();
        /*
                        tag: Option<String>,
                        author: Option<u32>,
                        multi_aids: &TagAidsLock,
                        article_lock: &ArticleCacheLock,
                        admin: Option<AdministratorCookie>, 
                        user: Option<UserCookie>, 
                        uhits: Option<UniqueHits>, 
                        gen: Option<GenTimer>, 
                        encoding: Option<AcceptCompression>,
                        msg: Option<String>,
        */
        // let output = cache::pages::rss::serve_filter();
        
        // if let Some(aids) = TagAidsLock.retrieve_tag_aids(u32) {
        //     if let Some(articles) = ArticleCacheLock.retrieve_articles(aids) {
        //         // return Some(articles);
        //     }
        // }
        // None
        // String::from("").into()
        
    }

    #[get("/rss.xml")]
    pub fn cache_rss(start: GenTimer,
                    text_lock: State<TextCacheLock>,
                    conn: DbConn,
                    // Removing user roles because they are not needed
                    // they are used for templates only
                    // and RSS feeds are currently Strings
                    // admin: Option<AdministratorCookie>,
                    // user: Option<UserCookie>,
                    encoding: AcceptCompression,
                    uhits: UniqueHits
                ) -> Express {
        
        let express: Express = cache::pages::rss::serve(&conn,
                                                        &*text_lock,
                                                        // admin,
                                                        // user,
                                                        None,
                                                        None,
                                                        Some(uhits),
                                                        Some(start.clone()),
                                                        Some(encoding),
                                                        None
                                                    );
        
        debug_timer(&start);
        express.compress( encoding )
    }
}


pub mod author {
    use super::*;

    #[get("/author/<author>/<authorname>")]
    pub fn cache_author_seo(start: GenTimer,
                            author: u32, 
                            authorname: &RawStr,
                            pagination: Page<Pagination>,
                            multi_aids: State<TagAidsLock>,
                            article_lock: State<ArticleCacheLock>,
                            conn: DbConn,
                            admin: Option<AdministratorCookie>,
                            user: Option<UserCookie>,
                            encoding: AcceptCompression,
                            uhits: UniqueHits
                           ) -> Express {
        routes::author::cache_author(start, author, pagination, multi_aids, article_lock, conn, admin, user, encoding, uhits)
    }

    #[get("/author/<author>")]
    pub fn cache_author(start: GenTimer,
                        author: u32,
                        pagination: Page<Pagination>,
                        multi_aids: State<TagAidsLock>,
                        article_lock: State<ArticleCacheLock>,
                        conn: DbConn,
                        admin: Option<AdministratorCookie>,
                        user: Option<UserCookie>,
                        encoding: AcceptCompression,
                        uhits: UniqueHits
                       ) -> Express {
        
        let express: Express = cache::pages::author::serve(author, 
                                                        &pagination, 
                                                        &conn, 
                                                        &multi_aids, 
                                                        &article_lock,
                                                        admin,
                                                        user,
                                                        Some(uhits),
                                                        Some(start.clone()),
                                                        Some(encoding),
                                                        None
                                                        );
        
        debug_timer(&start);
        express.compress( encoding )
        
    }
}

pub mod articles {
    use super::*;

    #[get("/")]
    pub fn cache_index(start: GenTimer, 
                       pagination: Page<Pagination>,
                       article_lock: State<ArticleCacheLock>,
                       num_articles: State<NumArticles>,
                       conn: DbConn, 
                       admin: Option<AdministratorCookie>, 
                       user: Option<UserCookie>, 
                       flash: Option<FlashMessage>, 
                       encoding: AcceptCompression, 
                       uhits: UniqueHits
                      ) -> Express 
    {
        let fmsg = fmsg(flash);
        
        let NumArticles(ref items_total) = *num_articles;
        let total_items = items_total.load(Ordering::SeqCst) as u32;
        let (ipp, cur_page, num_pages) = pagination.page_data(total_items);
        let info = pagination.page_info(total_items);
        
        // Define and build the welcome message
        let mut page_info: String;
        let welcome: &str = if cur_page == 1 {
            r##"<h1 style="text-align: center;">Welcome</h1>
<p>This is the personal blog of Andrew Prindle.  My recent topics of interest include:
the Rust programming language, web development, javascript, databases, cryptology, security, and compression.  
Feel free to contact me at the email address at the bottom of the page.</p>
<hr>
"##
        } else {
            r##"<h1 style="text-align: center;">Articles By Date</h1>
"##
        };
        page_info = String::with_capacity(info.len() + welcome.len() + 100);
        page_info.push_str( welcome );
        page_info.push_str( &info );
        
        let page_msg = Some(page_info);
        
        let express: Express = cache::pages::articles::serve(&*article_lock, 
                                                             pagination, 
                                                             &conn, 
                                                             admin, 
                                                             user, 
                                                             start.clone(), 
                                                             uhits, 
                                                             encoding, 
                                                             fmsg, 
                                                            //  page_info
                                                             page_msg
                                                            );
        
        debug_timer(&start);
        express.compress( encoding )
        
    }
}

































