use super::*;

#[get("/edit", rank=2)]
pub fn hbs_edit_unauthorized(start: GenTimer, user: Option<UserCookie>, encoding: AcceptCompression) -> Express {
    let mut loginmsg = String::with_capacity(300);
        loginmsg.push_str("You are not logged in, please <a href=\"");
        loginmsg.push_str(BLOG_URL);
        loginmsg.push_str("admin");
        loginmsg.push_str("\">Login</a>");
    
    let output = hbs_template(TemplateBody::General(alert_danger(&loginmsg)), None, Some("Unauthorized".to_string()), String::from("/edit"), None, user, None, Some(start.0));
    let express: Express = output.into();
    express.compress( encoding )
}

#[get("/edit/<aid>")]
pub fn hbs_edit(start: GenTimer, aid: u32, conn: DbConn, admin: AdministratorCookie, user: Option<UserCookie>, flash_opt: Option<FlashMessage>, encoding: AcceptCompression, uhits: UniqueHits) -> Express {
    let flash = process_flash(flash_opt);
    let cr_options = ComrakOptions { ext_header_ids: Some("section-".to_string()), .. COMRAK_OPTIONS };
    let output: Template;
    let id = ArticleId::new(aid);
    
    if let Some(mut article) = id.retrieve_with_conn(conn) {
        let title = article.title.clone();
        
        // If the body is empty that means the javascript did not process the Markdown into HTML
        // so convert the Markdown into HTML using Rust and the Comrak crate 
        //      The Comrak crate is slower than the pulldown-cmark but more options
        if &article.body == "" && &article.markdown != "" {
            let html: String = markdown_to_html(&article.markdown, &cr_options);
            article.body = html;
        }
        
        output = hbs_template(TemplateBody::Edit(EDIT_FORM_URL.to_string(), article), flash, Some(format!("Editing '{}'", title)), String::from("/edit"), Some(admin), user, None, Some(start.0));
        let express: Express = output.into();
        return express.compress(encoding);
    }
    
    output = hbs_template(TemplateBody::General("The reuqested article could not be found.".to_string()), flash, Some("Edit".to_string()), String::from("/edit"), Some(admin), user, None, Some(start.0));
    let express: Express = output.into();
    express.compress(encoding)
}


#[post("/edit", data = "<form>")]
pub fn hbs_edit_process(start: GenTimer, 
                        form: Form<ArticleWrapper>, 
                        conn: DbConn, 
                        article_cache: State<ArticleCacheLock>, 
                        multi_aids: State<TagAidsLock>,
                        num_articles: State<NumArticles>, 
                        text_cache: State<TextCacheLock>, 
                        admin: AdministratorCookie, 
                        encoding: AcceptCompression
                       ) -> Flash<Redirect> {
    
    let cr_options = ComrakOptions { ext_header_ids: Some("section-".to_string()), .. COMRAK_OPTIONS };
    
    let wrapper: ArticleWrapper = form.into_inner();
    let mut article: Article = wrapper.to_article();
    
    if &article.body == "" && &article.markdown != "" {
        let html: String = markdown_to_html(&article.markdown, &cr_options);
        article.body = html;
    }
    
    // println!("Processing Article info: {}", article.info());
    let result = article.update(&conn);
    match result {
        Ok(k) => {
            cache::update_all_caches(&conn, &*article_cache, &*multi_aids, &*num_articles, &*text_cache);
            
            Flash::success(Redirect::to(&format!("/edit/{}", &article.aid)), &k)
        },
        Err(ref e) if e == "" => {
            Flash::success(Redirect::to(&format!("/edit/{}", &article.aid)), &e)
        },
        Err(e) => {
            Flash::success(Redirect::to(&format!("/edit/{}", &article.aid)), &e)
        }
    }
}


