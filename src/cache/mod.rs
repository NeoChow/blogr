
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
use std::sync::atomic::{AtomicUsize, Ordering};

use std::borrow::Cow;

use evmap::*;
use comrak::{markdown_to_html, ComrakOptions};
use titlecase::titlecase;

use indexmap::IndexMap;


pub mod body;
use body::*;
pub mod pages;
use pages::*;

use super::*;
use blog::*;
use collate::*;
use content::*;
use data::*;
use templates::*;
use xpress::*;



pub fn make_descriptions(articles: Vec<Article>) -> Vec<Article> {
    let output: Vec<Article> = articles.into_iter()
        .map(|mut article| {
            article.body = if article.description != "" {
                article.description.clone()
            } else {
                article.body[..DESC_LIMIT].to_owned()
            };
            article
        }
    ).collect();
    output
}





pub struct NumArticles(pub AtomicUsize);






pub struct ArticleCacheLock {
    pub lock: RwLock<ArticleCache>,
}

pub struct ArticleCache {
    pub articles: IndexMap<u32, Article>,
}

impl ArticleCache {
    pub fn load_cache(conn: &DbConn) -> Self {
        if let Some(articles) = conn.articles_full("") {
            let mut map: IndexMap<u32, Article> = IndexMap::new();
            for article in articles {
                map.insert(article.aid, article);
            }
            ArticleCache{ articles: map }
        } else {
            ArticleCache{ articles: IndexMap::new() }
        }
    }
}

impl ArticleCacheLock {
    pub fn new(cache: ArticleCache) -> Self {
        ArticleCacheLock{ lock: RwLock::new( cache ) }
    }
    
    
    pub fn update_cache(&self, conn: &DbConn) -> Result<(), ()> {
        if let Ok(mut article_cache) = self.lock.write() {
            *article_cache = ArticleCache::load_cache(&conn);
            Ok( () )
        } else {
            println!("Failed to run update_articles() - could not acquire write lock");
            Err( () )
        }
    }
    
    pub fn num_articles(&self) -> u32 {
        if let Ok(article_cache) = self.lock.read() {
            article_cache.articles.len() as u32
        } else {
            0
        }
    }
    pub fn retrieve_article(&self, aid: u32) -> Option<Article> {
        if let Ok(article_cache) = self.lock.read() {
            if let Some(article) = article_cache.articles.get(&aid) {
                Some(article.clone())
            } else {
                None
            }
        } else {
            None
        }
    }
    pub fn retrieve_articles(&self, aids: Vec<u32>) -> Option<Vec<Article>> {
        if let Ok(article_cache) = self.lock.read() {
            let mut articles: Vec<Article> = Vec::new();
            for aid in aids {
                if let Some(article) = article_cache.articles.get(&aid) {
                    articles.push(article.clone());
                } else {
                    println!("Failed to retrieve article {} from collection", aid);
                }
            }
            if articles.len() != 0 {
                Some(articles)
            } else {
                None
            }
        } else {
            None
        }
    }
    pub fn all_articles<T: Collate>(&self, pagination: &Page<T>) -> Option<(Vec<Article>, u32)> {
        let mut starting = pagination.cur_page as u32;
        let mut ending = pagination.cur_page as u32 + pagination.settings.ipp() as u32;
        
        if let Ok(article_lock) = self.lock.read() {
            let aids: Vec<u32> = article_lock.articles.keys().map(|i| *i).collect();
            let total_items = aids.len() as u32;
            
            let mut starting = pagination.start();
            let mut ending = pagination.end();
            
            if total_items == 0 || starting >= total_items {
                return None;
            } else if ending >= total_items {
                ending = total_items - 1;
            }
            
            if total_items == 1 || pagination.settings.ipp() == 1 || ending.saturating_sub(starting) == 0 {
                if let Some(aid) = aids.get(starting as usize) {
                    if let Some(article) = self.retrieve_article(*aid) {
                        let mut art: Vec<Article> = vec![article];
                        if !ONE_RESULT_SHOW_FULL && (!ONE_RESULT_ONE_PAGE || (ONE_RESULT_ONE_PAGE && pagination.num_pages(total_items) == 1)) {
                            art = make_descriptions(art);
                        }
                        Some( ( art, total_items ) )
                    } else {
                        None
                    }
                } else {
                    None
                }
            } else {
                let slice: &[u32] = &aids[starting as usize..(ending+1) as usize];
                let ids = slice.to_owned();
                if let Some(mut articles) = self.retrieve_articles(ids) {
                    articles = make_descriptions(articles);
                    Some( (articles, total_items) )
                } else {
                    println!("Could not retrieve all articles page - retrieve_articles failed");
                    None
                }
            }
            
        } else {
            None
        }
    }
}



pub struct TextCacheLock {
    pub lock: RwLock<TextCache>,
}

pub struct TextCache {
    pub pages: HashMap<String, String>,
}
impl TextCache {
    pub fn load_cache(conn: &DbConn, multi_aids: &TagAidsLock) -> Self {
        let mut pages: HashMap<String, String> = HashMap::new();
        
        let rss = cache::pages::rss::load_rss(conn);
        pages.insert("rss".to_owned(), rss);
        
        TextCache {
            pages
        }
    }
}
impl TextCacheLock {
    pub fn new(cache: TextCache) -> Self {
        TextCacheLock{ lock: RwLock::new(cache) }
    }
    // For text retrieval maybe add a closure or function pointer parameter
    // that will be called in case the specified index(cached text) is not in the cache
    pub fn retrieve_text(&self, idx: &str) -> Option<String> {
        if let Ok(text_cache) = self.lock.read() {
            text_cache.pages.get(idx).map(|s| s.clone())
        } else {
            None
        }
    }
}





pub struct AidsCache {
    pub pages: HashMap<String, Vec<u32>>,
}
pub struct TagsCache {
    pub tags: Vec<TagCount>,
}

pub struct TagAidsLock {
    pub aids_lock: RwLock<AidsCache>,
    pub tags_lock: RwLock<TagsCache>,
}

impl TagsCache {
    pub fn load_cache(conn: &DbConn) -> Self {
        // Find all unique tags and store the number of times they are used
        // in a HashMap<String, u32>
        
        let qrystr = "SELECT COUNT(*) as cnt, unnest(tag) as untag FROM articles GROUP BY untag ORDER BY cnt DESC;";
        let qry = conn.query(qrystr, &[]);
        if let Ok(result) = qry {
            let mut tags: Vec<TagCount> = Vec::new();
            for row in &result {
                let c: i64 = row.get(0);
                let t: String = row.get(1);
                let t2: String = t.trim_matches('\'').to_string();
                let tc: TagCount = TagCount {
                    tag: titlecase(&t2),
                    url: t2,
                    count: c as u32,
                    size: 0,
                };
                tags.push(tc);
            }
            
            if tags.len() > 4 {
                if tags.len() > 7 {
                    let mut i = 0u16;
                    for mut v in &mut tags[0..6] {
                        v.size = 6-i;
                        i += 1;
                    }
                } else {
                    let mut i = 0u16;
                    for mut v in &mut tags[0..3] {
                        v.size = (3-i)*2;
                    }
                }
                tags.sort_by(|a, b| a.tag.cmp(&b.tag));
            }
            
            TagsCache {
                tags,
            }
        } else {
            TagsCache {
                tags: Vec::new(),
            }
        }
    }
}

impl TagAidsLock {
    // Returns the ArticleIds for the given page
    pub fn retrieve_aids(&self, page: &str) -> Option<Vec<u32>> {
        // unlock TagAidsLock
        // find the page
        // return the aids
        if let Ok(multi_aids) = self.aids_lock.read() {
            if let Some(aids) = multi_aids.pages.get(page) {
                Some(aids.clone())
            } else {
                None
            }
        } else {
            None
        }
    }
    // Retrieve (from the cache) all tags and the number of times they have been used
    pub fn retrieve_tags(&self) -> Option<Vec<TagCount>> {
        if let Ok(all_tags) = self.tags_lock.read() {
            Some(all_tags.tags.clone())
        } else {
            None
        }
    }
    
    pub fn load_cache(conn: &DbConn) -> Self {
        // Load tags then for each tag call 
        // cache::pages::tag::load_tag_aids(conn, tag) -> Option<Vec<u32>>
        // in order to find all articles attributed to that tag
        // 
        // Call cache::pages::author::load_authors(conn)
        // and call cache::pages::author::load_author_articles(conn, userid)
        // on each of the userids returned by the load_authors()
        
        let tag_cache = TagsCache::load_cache(&conn);
        let authors = cache::pages::author::load_authors(conn);
        
        let mut article_cache: HashMap<String, Vec<u32>> = HashMap::with_capacity(tag_cache.tags.len() + authors.len() + 10);
        
        for tag in tag_cache.tags.iter() {
            if let Some(aids) = cache::pages::tag::load_tag_aids(conn, &tag.tag) {
                let key = format!("tag/{}", &tag.tag.to_lowercase());
                if !PRODUCTION { println!("Loading tag {}\n\t{:?}\n\trelated articles:\n\t{:#?}", &tag.url, &tag, &aids); }
                article_cache.insert(key, aids);
            } else {
                println!("Error loading multi article cache on tag {} - no articles found", tag.url);
            }
        }
        
        for user in authors {
            if let Some(aids) = cache::pages::author::load_author_articles(conn, user) {
                let key = format!("author/{}", user);
                if !PRODUCTION { println!("Loading user {}\n\trelated articles:\n\t{:#?}", &user, &aids); }
                article_cache.insert(key, aids);
            } else {
                println!("Error loadign multi article cache on author {} - no articles found", user);
            }
        }
        
        TagAidsLock {
            aids_lock: RwLock::new( AidsCache{ pages: article_cache } ),
            tags_lock: RwLock::new( tag_cache ),
        }
        
    }
    #[inline]
    pub fn new(aids: AidsCache, tags: TagsCache) -> Self {
        TagAidsLock{ aids_lock: RwLock::new( aids), tags_lock: RwLock::new( tags ) }
    }
    
    pub fn multi_articles<T: Collate>(&self, article_cache: &ArticleCacheLock, multi_page: &str, pagination: &Page<T>) -> Option<(Vec<Article>, u32)> {
        if let Some(aids) = self.retrieve_aids(multi_page) {
            let total_items = aids.len() as u32;
            let mut starting = pagination.start();
            let mut ending = pagination.end();
            
            if total_items == 0 || starting >= total_items {
                return None;
            } else if ending >= total_items {
                ending = total_items - 1;
            }
            if total_items == 1 || pagination.settings.ipp() == 1 || ending.saturating_sub(starting) == 0 {
                if let Some(aid) = aids.get(starting as usize) {
                    if let Some(article) = article_cache.retrieve_article(*aid) {
                        let mut art: Vec<Article> = vec![article];
                        if !ONE_RESULT_SHOW_FULL && (!ONE_RESULT_ONE_PAGE || (ONE_RESULT_ONE_PAGE && pagination.num_pages(total_items) == 1)) {
                            art = make_descriptions(art);
                        }
                        Some( ( art, total_items ) )
                    } else {
                        None
                    }
                } else {
                    None
                }
            } else {
                let slice: &[u32] = &aids[starting as usize..(ending+1) as usize];
                let ids = slice.to_owned();
                if let Some(mut articles) = article_cache.retrieve_articles(ids) {
                    articles = make_descriptions(articles);
                    Some( (articles, total_items) )
                } else {
                    println!("Could not retrieve mult-articles page for `{}` - retrieve_articles failed", multi_page);
                    None
                }
            }
            
        } else {
            println!("Error retrieving multi-article page, retrieve_aids() failed for `{}`", multi_page);
            None
        }
        
    }
    pub fn author_articles<T: Collate>(&self, article_cache: &ArticleCacheLock, author: u32, pagination: &Page<T>) -> Option<(Vec<Article>, u32)> {
        let multi_page = format!("author/{}", author);
        self.multi_articles(article_cache, &multi_page, pagination)
    }
    pub fn tag_articles<T: Collate>(&self, article_cache: &ArticleCacheLock, tag: &str, pagination: &Page<T>) -> Option<(Vec<Article>, u32)> {
        let multi_page = format!("tag/{}", tag.to_lowercase());
        self.multi_articles(article_cache, &multi_page, pagination)
        
    }
}


pub fn template<T: BodyContext, U: BodyContext>(body_rst: Result<CtxBody<T>, CtxBody<U>>) -> Express where T: serde::Serialize, U: serde::Serialize {
    match body_rst {
        Ok(body)  => {
            let template = Template::render(T::template_name(), body);
            let express: Express = template.into();
            express
        },
        Err(body) => {
            let template = Template::render(U::template_name(), body);
            let express: Express = template.into();
            express
        },
    }
}


/*
    General - body text String
    Article - Article
    ArticlesPages - Vec<Article>, Page<Pagination>, u32 (total num items), Option<String> (page info - "Showing page x of y - z items found")
    Search - Vec<Article>, Option<Search>
    Login - String action url, Option<String> username
    LoginData - String action url, Option<String> username, HashMap<String, String> hidden form fields
    Create - String action url
    Edit - String action url, Article
    Manage - Vec<Article>, Page<Pagination>, u32 (num items total), Sort
    Tags - Vec<TagCount> lists tags and their counts
*/

pub fn load_all_articles(conn: &DbConn) -> Option<Vec<Article>> {
    if let Some(articles) = conn.articles_full("") {
        Some(articles)
    } else {
        None
    }
}

pub fn load_articles_map(conn: &DbConn) -> Option<HashMap<u32, Article>> {
    if let Some(articles) = conn.articles_full("") {
        let mut map: HashMap<u32, Article> = HashMap::new();
        for article in articles {
            map.insert(article.aid, article);
        }
        Some(map)
    } else {
        None
    }
}



pub fn update_article_caches(conn: &DbConn,
                             article_cache: &ArticleCacheLock, 
                             multi_aids: &TagAidsLock, 
                             num_articles: &NumArticles
                            ) -> bool {
    let mut output = true;
    if let Ok(mut article_cache) = article_cache.lock.write() {
        *article_cache = ArticleCache::load_cache(&conn);
    } else {
        println!("Failed to update_article_caches() - could not unlock article cache");
        output = false;
    }
    
    let multi = TagAidsLock::load_cache(&conn);
    let (tags_multi, aids_multi) = destruct_multi(multi);
    
    if let Ok(mut tags) = multi_aids.tags_lock.write() {
        *tags = tags_multi;
    } else {
        println!("Failed to update_article_caches() - could not unlock multi cache - tags");
        output = false;
    }
    
    if let Ok(mut multi_aids) = multi_aids.aids_lock.write() {
        *multi_aids = aids_multi;
    } else {
        println!("Failed to update_article_caches() - could not unlock multi cache - ArticleIds");
        output = false;
    }
    
    num_articles.0.store( article_cache.num_articles() as usize, Ordering::Relaxed );
    
    output
}


pub fn update_text_cache(conn: &DbConn, text_cache: &TextCacheLock, multi_aids: &TagAidsLock) -> bool {
    
    if let Ok(mut text_cache) = text_cache.lock.write() {
        *text_cache = TextCache::load_cache(&conn, &multi_aids);
        true
    } else {
        println!("Failed refresh content - could not unlock text cache");
        false
    }
    
}

// When an article is created, updated, or deleted all caches (except content caches)
//   must be updated, even the text cache; the text cache contains the RSS feed
//   which should be updated whenever the article cache is modified
#[inline]
pub fn update_all_caches(conn: &DbConn,
                         article_cache: &ArticleCacheLock, 
                         multi_aids: &TagAidsLock, 
                         num_articles: &NumArticles,
                         text_cache: &TextCacheLock,
                        ) -> bool 
{
    let mut output = true;
    if !update_article_caches(conn, article_cache, multi_aids, num_articles) {
        output = false;
    }
    
    if ! update_text_cache(conn, text_cache, multi_aids) {
        output = false;
    }
    
    output
}










