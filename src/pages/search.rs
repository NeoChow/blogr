use super::*;
    
#[get("/search?<search>")]
pub fn hbs_search_redirect(start: GenTimer, pagination: Page<Pagination>, search: Search, conn: DbConn, admin: Option<AdministratorCookie>, user: Option<UserCookie>, encoding: AcceptCompression) -> Redirect {
    // Add min/max date later, its not implemented in the search page anyways
    /* 
    let min = if let Some(mi) = search.min {
        format!("{}", mi.0.format("%Y-%m-%d %H:%M:%S"))
    } else {
        String::new()
    };
    let max = if let Some(mi) = search.min {
        format!("{}", mi.0.format("%Y-%m-%d %H:%M:%S"))
    } else {
        String::new()
    };
     */
    if let Some(q) = search.q {
        Redirect::to( &format!( "/search/{}", q ) )
    } else {
        Redirect::to( "/search" )
    }
}

#[get("/search/<searchstr>")]
pub fn hbs_search_results(start: GenTimer, pagination: Page<Pagination>, searchstr: String, conn: DbConn, admin: Option<AdministratorCookie>, user: Option<UserCookie>, encoding: AcceptCompression, uhits: UniqueHits) -> Express {
    
    let search = Search {
        limit: None,
        o: None,
        p: None,
        q: Some(searchstr),
        min: None,
        max: None,
    };
   
    // println!("Search parameters:\n{:?}", search);
    let mut countqry = String::with_capacity(750);
    let mut qrystr = String::with_capacity(750);
    
    // New Query:
    qrystr.push_str(r#"
SELECT a.aid, a.title, a.posted, 
    ts_headline('pg_catalog.english', a.body, fqry, 'StartSel = "<mark>", StopSel = "</mark>"') AS body, 
    a.tag, a.description, u.userid, u.display, u.username, 
    a.image, a.markdown, a.modified,
    ts_rank(a.fulltxt, fqry, 32) AS rank
FROM articles a JOIN users u ON (a.author = u.userid),
    plainto_tsquery('pg_catalog.english', '"#);
    
    countqry.push_str(r##"SELECT COUNT(*) FROM articles a, plainto_tsquery('pg_catalog.english', '"##);
    
    let mut wherestr = String::new();
    let original = search.clone();
    
    let mut tags: Option<String> = None;
    if let Some(mut q) = search.q {
        if &q != "" {
            // prevent attacks based on length and complexity of the sql query for full-text searches
            if q.len() > 200 {
                q = q[..200].to_string();
            }
            let sanitized = &sanitize_sql(q);
            qrystr.push_str(sanitized);
            countqry.push_str(sanitized);
            // do a full-text search on title, description, and body fields
            // for each word add: 'word' = ANY(tag)
            let ts = sanitized.split(" ").map(|s| format!("'{}' = ANY(a.tag)", s)).collect::<Vec<_>>().join(" OR ");
            tags = if &ts != "" { Some(ts) } else { None };
            // wherestr.push_str(&tags);
        }
    }
    qrystr.push_str("') fqry WHERE fqry @@ a.fulltxt");
    countqry.push_str("') fqry WHERE fqry @@ a.fulltxt");
    
    if let Some(t) = tags {
        qrystr.push_str(" OR ");
        qrystr.push_str(&t);
        // Inspect: Why is this repeated?? Should the last push(&t) be removed??
        countqry.push_str(" OR ");
        countqry.push_str(&t);
    }
    if let Some(min) = search.min {
        // after min
        qrystr.push_str(" AND a.posted > '");
        qrystr.push_str(&format!("{}", min.0.format("%Y-%m-%d %H:%M:%S")));
        qrystr.push_str("'");
        
        countqry.push_str(" AND a.posted > '");
        countqry.push_str(&format!("{}", min.0.format("%Y-%m-%d %H:%M:%S")));
        countqry.push_str("'");
    }
    if let Some(max) = search.max {
        // before max
        qrystr.push_str(" AND a.posted < '");
        qrystr.push_str(&format!("{}", max.0.format("%Y-%m-%d %H:%M:%S")));
        qrystr.push_str("'");
        
        countqry.push_str(" AND a.posted < '");
        countqry.push_str(&format!("{}", max.0.format("%Y-%m-%d %H:%M:%S")));
        countqry.push_str("'");
    }
    
    // println!("Generated the following SQL Query:\nCount:\n{}\n\nSearch Query:\n{}", countqry, qrystr);
    let total_query = countqry;
    let output: Template;
    if let Ok(rst) = conn.query(&total_query, &[]) {
        if !rst.is_empty() && rst.len() == 1 {
            let row = rst.get(0);
            let count: i64 = row.get(0);
            let total_items: u32 = count as u32;
            let (ipp, cur, num_pages) = pagination.page_data(total_items);
            let sql = pagination.sql(&qrystr, Some("rank DESC"));
            println!("Prepared paginated query:\n{}", sql);
            if let Some(results) = conn.articles(&sql) {
                if results.len() != 0 {
                    let pinfo = pagination.page_info(total_items);
                    let welcome = r##"<h1>Search Results</h1>"##;
                    
                    let mut page_information = String::with_capacity(pinfo.len() + welcome.len() + 50);
                    page_information.push_str(welcome);
                    page_information.push_str(&pinfo);
                    
                    output = hbs_template(TemplateBody::ArticlesPages(results, pagination, total_items, Some(page_information)), None, Some("Search Results".to_string()), String::from("/search"), admin, user, None, Some(start.0));
                    let express: Express = output.into();
                    // let end = start.0.elapsed();
                    // println!("Served in {}.{:09} seconds", end.as_secs(), end.subsec_nanos());
                    return express.compress( encoding );
                }
            }
        }
    }
    
    output = hbs_template(TemplateBody::General(alert_danger("No articles to show.")), None, Some("Search Results".to_string()), String::from("/search"), admin, user, None, Some(start.0));
    let express: Express = output.into();
    // let end = start.0.elapsed();
    // println!("Served in {}.{:09} seconds", end.as_secs(), end.subsec_nanos());
    express.compress( encoding )
    
    
}


