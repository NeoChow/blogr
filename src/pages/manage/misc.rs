use super::*;
#[get("/refresh_content")]
pub fn refresh_content(start: GenTimer, 
                       conn: DbConn,
                       article_cache: State<ArticleCacheLock>,
                       multi_aids: State<TagAidsLock>,
                       num_articles: State<NumArticles>,
                       text_cache: State<TextCacheLock>,
                       context_state: State<ContentContext>, 
                       cache_state: State<ContentCacheLock>,
                       admin: AdministratorCookie, 
                       user: Option<UserCookie>, 
                       encoding: AcceptCompression, 
                       uhits: UniqueHits
                      ) -> Express {
    
    cache::update_all_caches(&conn, &*article_cache, &*multi_aids, &*num_articles, &*text_cache);
    
    let mut ctx_writer;
    if let Ok(ctx) = context_state.pages.write() {
        ctx_writer = ctx;
    } else {
        let template = hbs_template(TemplateBody::General(alert_danger("An error occurred attempting to access content.")), None, Some("Content not available.".to_string()), String::from("/error404"), Some(admin), user, None, Some(start.0));
        let express: Express = template.into();
        return express.compress(encoding);
    }
    
    let mut cache_writer;
    if let Ok(cache) = cache_state.pages.write() {
        cache_writer = cache;
    } else {
        let template = hbs_template(TemplateBody::General(alert_danger("An error occurred attempting to access content.")), None, Some("Content not available.".to_string()), String::from("/error404"), Some(admin), user, None, Some(start.0));
        let express: Express = template.into();
        return express.compress(encoding);
    }
    
    let (ctx_pages, ctx_size) = destruct_context(ContentContext::load(STATIC_PAGES_DIR));
    *ctx_writer = ctx_pages;
    context_state.size.store(ctx_size, Ordering::SeqCst);
    
    let (cache_pages, cache_size) = destruct_cache(ContentCacheLock::new());
    *cache_writer = cache_pages;
    cache_state.size.store(cache_size, Ordering::SeqCst);
    
    let template = hbs_template(TemplateBody::General(alert_success("Content has been refreshed successfully.")), None, Some("Content refreshed.".to_string()), String::from("/error404"), Some(admin), user, None, Some(start.0));
    let express: Express = template.into();
    express.compress(encoding)
}

#[get("/backup", rank=2)]
pub fn hbs_backup_unauthorized(start: GenTimer, user: Option<UserCookie>, encoding: AcceptCompression) -> Express {
    let mut loginmsg = String::with_capacity(300);
        loginmsg.push_str("You are not logged in, please <a href=\"");
        loginmsg.push_str(BLOG_URL);
        loginmsg.push_str("admin");
        loginmsg.push_str("\">Login</a>");
    
    let output = hbs_template(TemplateBody::General(alert_danger(&loginmsg)), None, Some("Unauthorized".to_string()), String::from("/backup"), None, user, None, Some(start.0));
    let express: Express = output.into();
    express.compress( encoding )
}

#[get("/backup")]
pub fn backup(start: GenTimer, admin: AdministratorCookie, user: Option<UserCookie>, encoding: AcceptCompression, uhits: UniqueHits) -> Express {
    use std::process::Command;
    use rocket::http::hyper::header::{Headers, ContentDisposition, DispositionType, DispositionParam, Charset};
    
    let constr = format!("--dbname=\"{}\"", DATABASE_URL);
    
    #[cfg(not(production))]
    let output_rst = Command::new(DB_BACKUP_SCRIPT).output();
    #[cfg(production)]
    let output_rst = Command::new(DB_BACKUP_SCRIPT)
        .arg(DB_BACKUP_ARG)
        .output();
    
    if let Ok(output) = output_rst {
        let now = Local::now().naive_local();
        let today = now.date();
        
        let dl_name = now.format("db_blog_%Y-%m-%d.sql").to_string();
        
        let disposition = ContentDisposition {
            disposition: DispositionType::Attachment,
            parameters: vec![DispositionParam::Filename(
              Charset::Iso_8859_1, // The character set for the bytes of the filename
              None, // The optional language tag (see `language-tag` crate)
                dl_name.into_bytes() // b"db_blog-".to_vec() // the actual bytes of the filename
            )]
        };
        
        let backup = String::from_utf8_lossy(&output.stdout).into_owned();
        let length = backup.len();
        // println!("Backup succeeded with a length of {} bytes", length);
        let express: Express = backup.into();
        express.set_content_type(ContentType::Binary)
                .add_header(disposition)
    } else {
        let output = hbs_template(TemplateBody::General(alert_danger("Backup failed.")), None, Some("Backup Failed".to_string()), String::from("/backup"), Some(admin), user, None, Some(start.0));
        
        let express: Express = output.into();
        express.compress(encoding)
    }
}


