use super::*;

#[get("/user", rank = 1)]
pub fn hbs_dashboard_user_authorized(start: GenTimer, conn: DbConn, admin: Option<AdministratorCookie>, user: UserCookie, flash_msg_opt: Option<FlashMessage>, encoding: AcceptCompression) -> Express {
    let flash = if let Some(flash) = flash_msg_opt {
        Some( alert_warning(flash.msg()) )
    } else {
        None
    };
    
    let output: Template = hbs_template(TemplateBody::General(format!("Welcome User {user}.  You are viewing the User dashboard page.", user=user.username)), flash, Some("User Dashboard".to_string()), String::from("/user"), admin, Some(user), None, Some(start.0));
    
    let end = start.0.elapsed();
    // println!("Served in {}.{:09} seconds", end.as_secs(), end.subsec_nanos());
    let express: Express = output.into();
    express.compress(encoding)
}

#[get("/user", rank = 2)]
pub fn hbs_dashboard_user_flash(start: GenTimer, conn: DbConn, admin: Option<AdministratorCookie>, flash_msg_opt: Option<FlashMessage>, encoding: AcceptCompression) -> Express {
    let output: Template;
    
    if let Some(flash_msg) = flash_msg_opt {
        let flash = Some( alert_danger(flash_msg.msg()) );
        output = hbs_template(TemplateBody::Login(USER_LOGIN_URL.to_string(), None), flash, Some("User Login".to_string()), String::from("/user"), admin, None, Some("set_login_focus();".to_string()), Some(start.0));
    } else {
        output = hbs_template(TemplateBody::Login(USER_LOGIN_URL.to_string(), None), None, Some("User Login".to_string()), String::from("/user"), admin, None, Some("set_login_focus();".to_string()), Some(start.0));
    }
    
    // let end = start.0.elapsed();
    // println!("Served in {}.{:09} seconds", end.as_secs(), end.subsec_nanos());
    let express: Express = output.into();
    express.compress(encoding)
}


#[get("/user?<user>")]
pub fn hbs_dashboard_user_retry_user(start: GenTimer, conn: DbConn, admin: Option<AdministratorCookie>, mut user: QueryUser, flash_msg_opt: Option<FlashMessage>, encoding: AcceptCompression) -> Express {
    let username = if &user.user != "" { Some(user.user.clone() ) } else { None };
    let flash = if let Some(f) = flash_msg_opt { Some(alert_danger(f.msg())) } else { None };
    let output = hbs_template(TemplateBody::Login(USER_LOGIN_URL.to_string(), username), flash, Some("User Login".to_string()), String::from("/user"), admin, None, Some("set_login_focus();".to_string()), Some(start.0));
    
    // let end = start.0.elapsed();
    // println!("Served in {}.{:09} seconds", end.as_secs(), end.subsec_nanos());
    let express: Express = output.into();
    express.compress(encoding)
}

#[allow(unused_mut)]
#[post("/user", data = "<form>")]
pub fn hbs_process_user_login(start: GenTimer, form: Form<LoginCont<UserForm>>, admin: Option<AdministratorCookie>, mut cookies: Cookies) -> Result<Redirect, Flash<Redirect>> {
    let login: UserForm = form.get().form();
    let mut output = login.flash_redirect("/user", "/user", &mut cookies);
    
    if output.is_ok() {
        if let Some(admin_cookie) = admin {
            if &admin_cookie.username != &login.username {
                if let Ok(redir) = output {
                    let flash_message: Flash<Redirect> = Flash::error( 
                        redir, 
                        &format!("The administrator user {} has been logged out.  You cannot log in with two separate user accounts at once.", 
                            &admin_cookie.username
                        )
                    );
                    // Log the regular user out
                    // would use UserCookie::delete_cookie(cookies) but cookies already gets sent elsewhere
                    cookies.remove_private( Cookie::named( AdministratorCookie::cookie_id() ) );
                    
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

#[get("/user_logout")]
pub fn hbs_logout_user(admin: Option<UserCookie>, mut cookies: Cookies) -> Result<Flash<Redirect>, Redirect> {
    if let Some(_) = admin {
        // cookies.remove_private(Cookie::named(UserCookie::cookie_id()));
        UserCookie::delete_cookie(&mut cookies);
        Ok(Flash::success(Redirect::to("/"), "Successfully logged out."))
    } else {
        Err(Redirect::to("/user"))
    }
}


