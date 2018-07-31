use super::*;
    
#[get("/manage", rank=2)]
pub fn hbs_manage_unauthorized(start: GenTimer, user: Option<UserCookie>, encoding: AcceptCompression) -> Express {
    let mut loginmsg = String::with_capacity(300);
        loginmsg.push_str("You are not logged in, please <a href=\"");
        loginmsg.push_str(BLOG_URL);
        loginmsg.push_str("admin");
        loginmsg.push_str("\">Login</a>");
    
    let output = hbs_template(TemplateBody::General(alert_danger(&loginmsg)), None, Some("Unauthorized".to_string()), String::from("/manage"), None, user, None, Some(start.0));
    let express: Express = output.into();
    express.compress( encoding )
}


#[get("/manage")]
pub fn hbs_manage_basic(start: GenTimer, pagination: Page<Pagination>, conn: DbConn, admin: AdministratorCookie, user: Option<UserCookie>, flash: Option<FlashMessage>, encoding: AcceptCompression, uhits: UniqueHits) -> Express {
    hbs_manage_full(start, "".to_string(), "".to_string(), pagination, conn, admin, user, flash, encoding, uhits)
}

#[get("/manage/<sortstr>/<orderstr>")]
pub fn hbs_manage_full(start: GenTimer, sortstr: String, orderstr: String, pagination: Page<Pagination>, conn: DbConn, admin: AdministratorCookie, user: Option<UserCookie>, flash: Option<FlashMessage>, encoding: AcceptCompression, uhits: UniqueHits) -> Express {
    
    let output: Template;
    
    let fmsg: Option<String>;
    if let Some(flashmsg) = flash {
        if flashmsg.name() == "error" {
            fmsg = Some(alert_danger( flashmsg.msg() ));
        } else if flashmsg.name() == "warning" {
            fmsg = Some(alert_warning( flashmsg.msg() ));
        } else if flashmsg.name() == "success" {
            fmsg = Some(alert_success( flashmsg.msg() ));
        } else {
            fmsg = Some(alert_info( flashmsg.msg() ));
        }
    }  else {
        fmsg = None;
    }
    
    let mut total_query = "SELECT COUNT(*) AS count FROM articles";
    
    let mut page_query = "SELECT a.aid, a.title, a.posted, '' as body, a.tag, a.description, u.userid, u.display, u.username, a.image, a.markdown, a.modified FROM articles a JOIN users u ON (a.author = u.userid)";
    
    let order = match sortstr.to_lowercase().as_ref() {
        "date" if &orderstr == "desc" => "posted DESC",
        "date" if &orderstr == "asc" => "posted",
        "date" => "posted DESC",
        "title" if &orderstr == "desc" => "title DESC",
        "title" if &orderstr == "asc" => "title",
        "title" => "title",
        _ if &orderstr == "desc" => "posted DESC",
        _ if &orderstr == "asc" => "posted",
        _ => "posted DESC",
    };
    
    let sort_options: Sort = match order {
        "posted DESC" => Sort::Date(true),
        "posted" => Sort::Date(false),
        "title" => Sort::Title(false),
        "title DESC" => Sort::Title(true),
        _ => Sort::Date(true),
    };
    
    if let Ok(rst) = conn.query(total_query, &[]) {
        if !rst.is_empty() && rst.len() == 1 {
            let countrow = rst.get(0);
            let count: i64 = countrow.get(0);
            let total_items: u32 = count as u32;
            let (ipp, cur, num_pages) = pagination.page_data(total_items);
            let pagesql = pagination.sql(page_query, Some(order));
            
            // println!("Generated pagination sql:\n{}\n", pagesql);
            
            if let Some(results) = conn.articles(&pagesql) {
                if results.len() != 0 {
                    output = hbs_template(TemplateBody::Manage(results, pagination, total_items, sort_options), fmsg, Some(format!("Manage Articles - Page {} of {}", cur, num_pages)), String::from("/manage"), Some(admin), user, None, Some(start.0));
                    
                    let express: Express = output.into();
                    return express.compress(encoding);
                }
            }
        }
    }

    output = hbs_template(TemplateBody::General(alert_danger("No articles found.")), fmsg, Some("Manage Articles".to_string()), String::from("/manage"), Some(admin), user, None, Some(start.0));
    let express: Express = output.into();
    express.compress(encoding)
}