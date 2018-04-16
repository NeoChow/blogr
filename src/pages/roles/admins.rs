use super::*;
    
#[get("/admin", rank = 1)]
pub fn hbs_dashboard_admin_authorized(start: GenTimer, pagination: Page<Pagination>, conn: DbConn, user: Option<UserCookie>, admin: AdministratorCookie, flash_msg_opt: Option<FlashMessage>, encoding: AcceptCompression, uhits: UniqueHits) -> Express {
    // let end = start.0.elapsed();
    // println!("Served in {}.{:09} seconds", end.as_secs(), end.subsec_nanos());
    // hbs_manage_full(start, "".to_string(), "".to_string(), pagination, conn, admin, user, flash_msg_opt, encoding, uhits)
    roles::manage::dashboard::hbs_manage_full(start, "".to_string(), "".to_string(), pagination, conn, admin, user, flash_msg_opt, encoding, uhits)
}

#[get("/admin", rank = 7)]
pub fn hbs_dashboard_admin_flash(start: GenTimer, conn: DbConn, user: Option<UserCookie>, flash_msg_opt: Option<FlashMessage>, encoding: AcceptCompression, referrer: Referrer) -> Express {
    let output: Template;
    
    let mut fields: HashMap<String, String> = HashMap::new();
    
    if let Referrer(Some(refer)) = referrer {
        // println!("Referrer: {}", &refer);
        fields.insert("referrer".to_string(), refer);
    } else {
        // println!("No referrer");
    }
    
    if let Some(flash_msg) = flash_msg_opt {
        let flash = Some( alert_danger(flash_msg.msg()) );
        output = hbs_template(TemplateBody::LoginData(ADMIN_LOGIN_URL.to_string(), None, fields), flash, Some("Administrator Login".to_string()), String::from("/admin"), None, user, Some("set_login_focus();".to_string()), Some(start.0));
    } else {
        output = hbs_template(TemplateBody::LoginData(ADMIN_LOGIN_URL.to_string(), None, fields), None, Some("Administrator Login".to_string()), String::from("/admin"), None, user, Some("set_login_focus();".to_string()), Some(start.0));
    }
    
    // let end = start.0.elapsed();
    // println!("Served in {}.{:09} seconds", end.as_secs(), end.subsec_nanos());
    let express: Express = output.into();
    express.compress(encoding)
}

#[get("/admin?<userqry>", rank=4)]
pub fn hbs_dashboard_admin_retry_user(start: GenTimer, conn: DbConn, user: Option<UserCookie>, mut userqry: QueryUser, flash_opt: Option<FlashMessage>, encoding: AcceptCompression) -> Express {
    let flash = process_flash(flash_opt);
    
    let username = if &userqry.user != "" { Some(userqry.user.clone() ) } else { None };
    let output = hbs_template(TemplateBody::Login(ADMIN_LOGIN_URL.to_string(), username), flash, Some("Administrator Login".to_string()), String::from("/admin"), None, user, Some("set_login_focus();".to_string()), Some(start.0));
    
    // let end = start.0.elapsed();
    // println!("Served in {}.{:09} seconds", end.as_secs(), end.subsec_nanos());
    let express: Express = output.into();
    express.compress(encoding)
}

#[get("/admin?<rediruser>", rank = 2)]
pub fn hbs_dashboard_admin_retry_redir(start: GenTimer, conn: DbConn, user: Option<UserCookie>, mut rediruser: QueryUserRedir, flash_opt: Option<FlashMessage>, encoding: AcceptCompression) -> Express {
    let flash = process_flash(flash_opt);
    
    let mut fields: HashMap<String, String> = HashMap::new();
    
    if &rediruser.referrer != "" && &rediruser.referrer != "noredirect" {
        // println!("Adding referrer {}", &rediruser.referrer);
        fields.insert("referrer".to_string(), rediruser.referrer.clone());
    } else {
        // println!("No referring page\n{:?}", rediruser);
    }
    
    let username = if &rediruser.user != "" { Some(rediruser.user.clone() ) } else { None };
    let output = hbs_template(TemplateBody::LoginData(ADMIN_LOGIN_URL.to_string(), username, fields), flash, Some("Administrator Login".to_string()), String::from("/admin"), None, user, Some("set_login_focus();".to_string()), Some(start.0));
    
    // let end = start.0.elapsed();
    // println!("Served in {}.{:09} seconds", end.as_secs(), end.subsec_nanos());
    let express: Express = output.into();
    express.compress(encoding)
}

#[get("/admin?<rediruser>", rank = 3)]
pub fn hbs_dashboard_admin_retry_redir_only(start: GenTimer, conn: DbConn, user: Option<UserCookie>, mut rediruser: QueryRedir, flash_opt: Option<FlashMessage>, encoding: AcceptCompression) -> Express {
    let flash = process_flash(flash_opt);
    
    let mut fields: HashMap<String, String> = HashMap::new();
    
    if &rediruser.referrer != "" && &rediruser.referrer != "noredirect" {
        // println!("Adding referrer {}", &rediruser.referrer);
        fields.insert("referrer".to_string(), rediruser.referrer.clone());
    } else {
        // println!("No referring page\n{:?}", rediruser);
    }
    
    let username = None;
    let output = hbs_template(TemplateBody::LoginData(ADMIN_LOGIN_URL.to_string(), username, fields), flash, Some("Administrator Login".to_string()), String::from("/admin"), None, user, Some("set_login_focus();".to_string()), Some(start.0));
    
    // let end = start.0.elapsed();
    // println!("Served in {}.{:09} seconds", end.as_secs(), end.subsec_nanos());
    let express: Express = output.into();
    express.compress(encoding)
}

#[allow(unused_mut)]
#[post("/admin", data = "<form>")]
pub fn hbs_process_admin_login(start: GenTimer, form: Form<LoginCont<AdministratorForm>>, user: Option<UserCookie>, mut cookies: Cookies) -> Result<Redirect, Flash<Redirect>> {
    let login: AdministratorForm = form.get().form();
    let mut err_temp: String;
    let ok_addy: &str;
    let err_addy: &str;
    if &login.referrer != "" && &login.referrer != "noredierct" {
        // println!("Processing referrer: {}", &login.referrer);
        let referring = if login.referrer.starts_with(BLOG_URL) {
            &login.referrer[BLOG_URL.len()-1..]
        } else {
            &login.referrer
        };
        ok_addy = &referring;
        err_addy = {
            err_temp = String::with_capacity(referring.len() + 20);
            err_temp.push_str("/admin?redir=");
            err_temp.push_str(referring);
            &err_temp
        };
    } else {
        ok_addy = "/admin";
        err_addy = "/admin";
    }
    let mut output = login.flash_redirect(ok_addy, err_addy, &mut cookies);
    
    if output.is_ok() {
        // println!("Login success, forwarding to {}", ok_addy);
        if let Some(user_cookie) = user {
            if &user_cookie.username != &login.username {
                if let Ok(redir) = output {
                    let flash_message: Flash<Redirect> = Flash::error( 
                        redir, 
                        &format!("The regular user {} has been logged out.  You cannot log in with two separate user accounts at once.", 
                            &user_cookie.username
                        )
                    );
                    // Log the regular user out
                    // would use UserCookie::delete_cookie(cookies) but cookies already gets sent elsewhere
                    cookies.remove_private( Cookie::named( UserCookie::cookie_id() ) );
                    
                    // the Err will still allow the cookies to get set to log the user in but will allow a message to be passed
                    output = Err( flash_message );
                }
            }
        }
    }
    
    // let end = start.0.elapsed();
    // println!("Processed in {}.{:09} seconds", end.as_secs(), end.subsec_nanos());
    output
}

#[get("/admin_logout")]
pub fn hbs_logout_admin(admin: Option<AdministratorCookie>, mut cookies: Cookies) -> Result<Flash<Redirect>, Redirect> {
    if let Some(_) = admin {
        // cookies.remove_private(Cookie::named(AdministratorCookie::cookie_id()));
        AdministratorCookie::delete_cookie(&mut cookies);
        Ok(Flash::success(Redirect::to("/"), "Successfully logged out."))
    } else {
        Err(Redirect::to("/admin"))
    }
}

