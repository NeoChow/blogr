use super::*;

#[post("/create", data = "<form>")]
pub fn hbs_article_process(start: GenTimer, 
                           form: Form<ArticleForm>, 
                           conn: DbConn, 
                           article_cache: State<ArticleCacheLock>, 
                           multi_aids: State<TagAidsLock>,
                           num_articles: State<NumArticles>, 
                           text_cache: State<TextCacheLock>, 
                           admin: AdministratorCookie, 
                           user: Option<UserCookie>, 
                           encoding: AcceptCompression
                          ) -> Express {
    let output: Template;
    let username = if let Some(ref display) = admin.display { display.clone() } else { titlecase(&admin.username) };
    let cr_options = ComrakOptions { ext_header_ids: Some("section-".to_string()), .. COMRAK_OPTIONS };
    let mut article: ArticleForm = form.into_inner();
    
    if &article.body == "" && &article.markdown != "" {
        let html: String = markdown_to_html(&article.markdown, &cr_options);
        article.body = html;
    }
    
    let result = article.save(&conn, admin.userid, &username);
    match result {
        Ok(article) => {
            cache::update_all_caches(&conn, &*article_cache, &*multi_aids, &*num_articles, &*text_cache);
            let title = article.title.clone();
            output = hbs_template(TemplateBody::Article(article), None, Some(title), String::from("/create"), Some(admin), user, None, Some(start.0));
        },
        Err(why) => {
            output = hbs_template(TemplateBody::General(alert_danger(&format!("Could not post the submitted article.  Reason: {}", why))), None, Some("Could not post article".to_string()), String::from("/create"), Some(admin), user, None, Some(start.0));
        },
    }
    
    // let end = start.0.elapsed();
    // println!("Served in {}.{:09} seconds", end.as_secs(), end.subsec_nanos());
    let express: Express = output.into();
    express.compress(encoding)
}
#[post("/create", rank=2)]
pub fn hbs_create_unauthorized(start: GenTimer, conn: DbConn, admin: Option<AdministratorCookie>, user: Option<UserCookie>, encoding: AcceptCompression) -> Express {
    let mut loginmsg = String::with_capacity(300);
        loginmsg.push_str("You are not logged in, please <a href=\"");
        loginmsg.push_str(BLOG_URL);
        loginmsg.push_str("admin");
        loginmsg.push_str("\">Login</a>");
    
    let output = hbs_template(TemplateBody::General(alert_danger(&loginmsg)), None, Some("Create".to_string()), String::from("/create"), None, user, None, Some(start.0));
    
    // let end = start.0.elapsed();
    // println!("Served in {}.{:09} seconds", end.as_secs(), end.subsec_nanos());
    let express: Express = output.into();
    express.compress(encoding)
}

#[get("/create")]
pub fn hbs_create_form(start: GenTimer, conn: DbConn, admin: Option<AdministratorCookie>, user: Option<UserCookie>, encoding: AcceptCompression, uhits: UniqueHits) -> Express {
    let output: Template;
    if admin.is_some() {
        output = hbs_template(TemplateBody::Create(CREATE_FORM_URL.to_string()), None, Some("Create New Article".to_string()), String::from("/create"), admin, user, None, Some(start.0));
    } else {
        output = hbs_template(TemplateBody::General(alert_danger(UNAUTHORIZED_POST_MESSAGE)), None, Some("Not Authorized".to_string()), String::from("/create"), admin, user, None, Some(start.0));
    }
    
    // let end = start.0.elapsed();
    // println!("Served in {}.{:09} seconds", end.as_secs(), end.subsec_nanos());
    let express: Express = output.into();
    express.compress(encoding)
}


