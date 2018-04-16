use super::*;
    
#[get("/pageviews", rank=2)]
pub fn hbs_pageviews_unauthorized(start: GenTimer, user: Option<UserCookie>, encoding: AcceptCompression) -> Express {
    let mut loginmsg = String::with_capacity(300);
        loginmsg.push_str("You are not logged in, please <a href=\"");
        loginmsg.push_str(BLOG_URL);
        loginmsg.push_str("admin");
        loginmsg.push_str("\">Login</a>");
    
    let output = hbs_template(TemplateBody::General(alert_danger(&loginmsg)), None, Some("Unauthorized".to_string()), String::from("/pageviews"), None, user, None, Some(start.0));
    let express: Express = output.into();
    express.compress( encoding )
}

#[get("/pagestats", rank=2)]
pub fn hbs_pagestats_unauthorized(start: GenTimer, user: Option<UserCookie>, encoding: AcceptCompression) -> Express {
    let mut loginmsg = String::with_capacity(300);
        loginmsg.push_str("You are not logged in, please <a href=\"");
        loginmsg.push_str(BLOG_URL);
        loginmsg.push_str("admin");
        loginmsg.push_str("\">Login</a>");
    
    let output = hbs_template(TemplateBody::General(alert_danger(&loginmsg)), None, Some("Unauthorized".to_string()), String::from("/pagestats"), None, user, None, Some(start.0));
    let express: Express = output.into();
    express.compress( encoding )
}

#[get("/pagestats")]
pub fn hbs_pagestats_no_errors(start: GenTimer, admin: AdministratorCookie, user: Option<UserCookie>, encoding: AcceptCompression, uhits: UniqueHits, counter_state: State<Counter>, unique_state: State<UStatsWrapper>) -> Redirect {
    Redirect::to("/pagestats/false")
}

#[get("/pagestats/<show_errors>")]
pub fn hbs_pagestats(start: GenTimer, show_errors: bool, admin: AdministratorCookie, user: Option<UserCookie>, encoding: AcceptCompression, uhits: UniqueHits, counter_state: State<Counter>, unique_state: State<UStatsWrapper>) -> Express {
    use urlencoding::decode;
    use htmlescape::*;
    
    let output: Template;
    let counter_mutex = counter_state.stats.lock();
    if let Ok(counter) = counter_mutex {
        let unique_rwlock = unique_state.0.read();
        if let Ok(unique) = unique_rwlock {
            let mut buffer = String::with_capacity((unique.stats.len() * 1000) + 1000);
            
            buffer.push_str(r#"<div class="v-stats-container-totals container">
                <div class="v-stats v-stats-total row">
                <div class="v-stats-page col-md">
                <i class="fa fa-bar-chart" aria-hidden="true"></i> Total Hits"#);
            buffer.push_str("<br>");
            if show_errors {
                buffer.push_str(r#"<a href=""#);
                buffer.push_str(&BLOG_URL);
                buffer.push_str(r#"pagestats/false">Show statistics for normal pages</a>"#);
            } else {
                buffer.push_str(r#"<a href=""#);
                buffer.push_str(&BLOG_URL);
                buffer.push_str(r#"pagestats/true">Show statistics for error pages</a>"#);
            }
            buffer.push_str(r#"</div><div class="v-stats-hits col-md">"#);
            buffer.push_str( &((&uhits.0).2.to_string()) );
            buffer.push_str(&format!(r#"<br><a href="{url}download/page_stats.json">Download basic log</a><br><a href="{url}download/unique_stats.json">Download IP Logs</a>"#, url=BLOG_URL));
            buffer.push_str(r#"</div></div></div><div class="v-stats-container container">"#);
            
            for (page, hits) in counter.map.iter() {
                // if page is an error page and show_errors is false, skip the page
                if page.starts_with("error") || page.starts_with("!error") {
                    if !show_errors { continue; }
                // if page is not an error page but show_errors is true, skip the entry
                } else if show_errors {
                    continue;
                }
                
                if let Some(u_stats) = unique.stats.get(page) {
                    // Extended statistics are available
                    let total_ips: usize = u_stats.len();
                    let total_visits: usize = u_stats.values().sum();
                    let avg_visits: usize = total_visits / total_ips;
                    
                    buffer.push_str(r#"<div class="v-stats row v-stats-extended"><div class="v-stats-page col">"#);
                    buffer.push_str( &encode_minimal( &decode(&page).unwrap_or("DECODE ERROR".to_owned()) ) );
                    buffer.push_str(r#"</div></div><div class="v-stats row"><div class="v-stats-hits col-auto"></div><div class="v-stats-hits col-lg-2" data-toggle="tooltip" data-html="false" title="Total hits">Hits: "#);
                    buffer.push_str(&hits.to_string());
                    buffer.push_str(r#"</div><div class="v-stats-hits col-lg-3" data-toggle="tooltip" data-html="false" title="Unique Visitors">Visitors: "#);
                    buffer.push_str(&total_ips.to_string());
                    buffer.push_str(r#"</div><div class="v-stats-hits col-lg-3" data-toggle="tooltip" data-html="false" title="Average Visits Per Unique Visitor">Avg Visits: "#);
                    buffer.push_str(&avg_visits.to_string());
                    buffer.push_str(r#"</div><div class="v-stats-hits col-lg-3" data-toggle="tooltip" data-html="false" title="Total Visits For Unique Visitors">Total Visits: "#);
                    buffer.push_str(&total_visits.to_string());
                    buffer.push_str(r#"</div></div>"#);
                } else {
                    buffer.push_str(r#"<div class="v-stats row v-stats-basic"><div class="v-stats-page col">"#);
                    buffer.push_str( &encode_minimal( &decode(&page).unwrap_or("DECODE ERROR".to_owned()) ) );
                    buffer.push_str(r#"</div></div><div class="v-stats row"><div class="v-stats-hits col-auto"></div><div class="v-stats-hits col-11" data-toggle="tooltip" data-html="false" title="Total hits">Hits: "#);
                    buffer.push_str(&hits.to_string());
                    buffer.push_str(r#"</div></div>"#);
                }
                
            }
            
            buffer.push_str("</div>");
            if buffer.capacity() > ((unique.stats.len() * 1000) + 1000) {
                println!("Pagestats buffer increased, original size: {}\n\tNew size: {}, new capacity: {}", ((unique.stats.len() * 1000) + 1000), buffer.len(), buffer.capacity());
            }
            output = hbs_template(TemplateBody::General(buffer), None, Some("Page Statistics".to_string()), String::from("/pagestats"), Some(admin), user, None, Some(start.0));
        } else {
            output = hbs_template(TemplateBody::General(alert_danger("Could not retrieve page statistics.<br>Failed to acquire read lock.")), None, Some("Page Views".to_string()), String::from("/pagestats"), Some(admin), user, None, Some(start.0));
        }
        
    } else {
        output = hbs_template(TemplateBody::General(alert_danger("Could not retrieve page statistics.<br>Failed to acquire mutex lock.")), None, Some("Page Views".to_string()), String::from("/pagestats"), Some(admin), user, None, Some(start.0));
    }
    
    // let end = start.0.elapsed();
    // println!("Served in {}.{:09} seconds", end.as_secs(), end.subsec_nanos());
    
    let express: Express = output.into();
    express.compress(encoding)
}

#[get("/pagestats2")]
pub fn hbs_pagestats2(start: GenTimer, admin: AdministratorCookie, user: Option<UserCookie>, encoding: AcceptCompression, uhits: UniqueHits, counter_state: State<Counter>, unique_state: State<UStatsWrapper>) -> Express {
    use urlencoding::decode;
    use htmlescape::*;
    
    let output: Template;
    let stats_lock = counter_state.stats.lock();
    if let Ok(counter) = stats_lock {
        let unique_lock = unique_state.0.read();
        if let Ok(unique_unlocked) = unique_lock {
            let unique = unique_unlocked;
            let mut buffer = String::with_capacity(unique.stats.len() * 500);
            
            for (page, ips) in unique.stats.iter() {
                let total_ips: usize = ips.len();
                let mut total_visits: usize = ips.values().sum();
                let avg_visits: usize = total_visits / total_ips;
                
                let hits: String = if let Some(h) = (*counter).map.get(page) {
                    h.to_string()
                } else {
                    "-".to_owned()
                };
                
                buffer.push_str(r#"<div class="v-stats row"><div class="v-stats-page col">"#);
                buffer.push_str(&page);
                buffer.push_str(r#"</div></div><div class="v-stats row"><div class="v-stats col-1"></div><div class="v-stats-hits col-3" data-toggle="tooltip" data-html="false" title="Total hitsr">Hits: "#);
                buffer.push_str(&hits);
                buffer.push_str(r#"</div><div class="v-stats-hits col-4" data-toggle="tooltip" data-html="false" title="Unique Visitors">Visitors: "#);
                buffer.push_str(&total_ips.to_string());
                buffer.push_str(r#"</div><div class="v-stats-hits col-4" data-toggle="tooltip" data-html="false" title="Average Visits Per Unique Visitor">Avg Visits: "#);
                buffer.push_str(&avg_visits.to_string());
                buffer.push_str(r#"</div></div>"#);
            }
            
            let mut page = String::with_capacity(buffer.len() + 500);
            
            page.push_str(r#"<div class="v-stats-container-totals container"><div class="v-stats v-stats-total row"><div class="v-stats-page col"><i class="fa fa-bar-chart" aria-hidden="true"></i> Total Hits</div><div class="v-stats-hits col-auto">"#);
            page.push_str( &((&uhits.0).2.to_string()) );
            page.push_str(r#"</div></div></div><div class="v-stats-container container">"#);
            page.push_str( &buffer );
            page.push_str("</div>");
            
            output = hbs_template(TemplateBody::General(page), None, Some("Page Statistics".to_string()), String::from("/pagestats"), Some(admin), user, None, Some(start.0));
            
        } else {
            output = hbs_template(TemplateBody::General(alert_danger("Could not retrieve page statistics.<br>Failed to acquire read lock.")), None, Some("Page Views".to_string()), String::from("/pagestats"), Some(admin), user, None, Some(start.0));
        }
    } else {
        output = hbs_template(TemplateBody::General(alert_danger("Could not retrieve page statistics.<br>Failed to acquire mutex lock.")), None, Some("Page Views".to_string()), String::from("/pagestats"), Some(admin), user, None, Some(start.0));
        
    }
    
    // let end = start.0.elapsed();
    // println!("Served in {}.{:09} seconds", end.as_secs(), end.subsec_nanos());
    
    let express: Express = output.into();
    express.compress(encoding)
}

#[get("/pageviews")]
pub fn hbs_pageviews(start: GenTimer, admin: AdministratorCookie, user: Option<UserCookie>, encoding: AcceptCompression, uhits: UniqueHits, stats: State<Counter>) -> Express {
    use urlencoding::decode;
    use htmlescape::*;
    
    let output: Template;
    let lockstats = stats.stats.lock();
    if let Ok(counter) = lockstats {
        
        let statistics: Vec<String> = counter.map.iter()
            .map(|(n, v)| 
                format!(r#"<div class="v-stats row"><div class="v-stats-page col">{}</div><div class="v-stats-hits col-auto">{}</div></div>"#, 
                    encode_minimal(&decode(n).unwrap_or(String::new()) ), v))
            .collect();
        
        
        let pages = statistics.join("\n");
        let mut page: String = String::with_capacity(pages.len() + 250);
        page.push_str(r#"<div class="v-stats-container-totals container"><div class="v-stats v-stats-total row"><div class="v-stats-page col"><i class="fa fa-bar-chart" aria-hidden="true"></i> Total Hits</div><div class="v-stats-hits col-auto">"#);
        page.push_str(&((&uhits.0).2.to_string()));
        page.push_str(r#"</div></div></div><div class="v-stats-container container">"#);
        page.push_str(&pages);
        page.push_str(r#"</div>"#);
        
        output = hbs_template(TemplateBody::General(page), None, Some("Page Views".to_string()), String::from("/pageviews"), Some(admin), user, None, Some(start.0));
    } else {
        output = hbs_template(TemplateBody::General(alert_danger("Could not retrieve page statistics.<br>Failed to acquire mutex lock.")), None, Some("Page Views".to_string()), String::from("/pageviews"), Some(admin), user, None, Some(start.0));
    }
    
    let express: Express = output.into();
    express.compress(encoding)
    
}


