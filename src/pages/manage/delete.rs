use super::*;

#[get("/delete", rank=2)]
pub fn hbs_delete_unauthorized(start: GenTimer, user: Option<UserCookie>, encoding: AcceptCompression) -> Express {
    let mut loginmsg = String::with_capacity(300);
        loginmsg.push_str("You are not logged in, please <a href=\"");
        loginmsg.push_str(BLOG_URL);
        loginmsg.push_str("admin");
        loginmsg.push_str("\">Login</a>");
    
    let output = hbs_template(TemplateBody::General(alert_danger(&loginmsg)), None, Some("Unauthorized".to_string()), String::from("/delete"), None, user, None, Some(start.0));
    let express: Express = output.into();
    express.compress( encoding )
}

#[get("/delete/<aid>")]
pub fn hbs_delete_confirm(start: GenTimer, aid: u32, conn: DbConn, admin: AdministratorCookie, user: Option<UserCookie>, encoding: AcceptCompression) -> Express {
    
    let confirm = alert_warning(&format!(r#"
        You are attempting to permanently delete an article, are you sure you want to continue?  
        This action cannot be undone.
        <form action="{}process_delete/{}" method="post" id="delete-form">
            <input type="hidden" value="{}" id="manage-page">
            <div class="v-centered-text">
                <button type="submit" id="delete-button" class="v-del-confirm btn btn-danger">Delete</button>
                <span class="v-spacer-del"></span>
                <button type="button" id="delete-cancel" class="v-del-cancel btn btn-warning">Cancel</button>
            </div>
        </form>
        "#, BLOG_URL, aid, MANAGE_URL));
    
    let output = hbs_template(TemplateBody::General(confirm), None, Some("Delete Article".to_string()), String::from("/delete"), Some(admin), user, None, Some(start.0));
    
    let express: Express = output.into();
    express.compress( encoding )
}

#[post("/process_delete/<aid>")]
pub fn hbs_process_delete(aid: u32, 
                          conn: DbConn, 
                          article_cache: State<ArticleCacheLock>, 
                          multi_aids: State<TagAidsLock>,
                          num_articles: State<NumArticles>, 
                          text_cache: State<TextCacheLock>, 
                          admin: AdministratorCookie, 
                          user: Option<UserCookie>
                         ) -> Result<Flash<Redirect>, Redirect> {
    let qrystr = format!("DELETE FROM articles WHERE aid = {}", aid);
    // println!("Delete query:\n{}\n", &qrystr);
    
    if let Ok(num) = conn.execute(&qrystr, &[]) {
        if num == 1 {
            // println!("Delete succeeded");
            cache::update_all_caches(&conn, &*article_cache, &*multi_aids, &*num_articles, &*text_cache);
            Ok( Flash::success(Redirect::to("/manage"), &format!("Article {} successfully deleted.", aid)) )
        } else if num == 0 {
            println!("Delete failed - no articles deleted.");
            Ok( Flash::error(Redirect::to("/manage"), &format!("Article {} was not deleted.", aid)) )
        } else {
            println!("Delete failed - multiple articles deleted!");
            Ok( Flash::error(Redirect::to("/manage"), &format!("A mistake occurred. Multiple articles ({} articles) appear to have been deleted.", num)) )
        }
    } else {
        println!("Delete failed.");
        Err( Redirect::to("/manage") )
    }
}

