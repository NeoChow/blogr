

use rocket::response::Flash;
use rocket::request::FlashMessage;
use rocket::response::content::Html;
use blog::*;
use titlecase::titlecase;

use super::{BLOG_URL, USER_LOGIN_URL, ADMIN_LOGIN_URL};


pub const UNAUTHORIZED_POST_MESSAGE: &'static str = "You are not authorized to post articles.  Please login as an administrator.<br><a href=\"admin\">Admin Login</a>";


const GENERIC_PAGE_START: &'static str = "<div class=\"v-content\">\n\t\t\t\t\t\t";
const GENERIC_PAGE_END: &'static str = "\n\t\t\t\t\t</div>";
const TABS: &'static str = "\t\t\t\t\t\t\t";


pub fn process_flash(flash_opt: Option<FlashMessage>) -> Option<String> {
    let fmsg: Option<String>;
    if let Some(flash) = flash_opt {
        if flash.name() == "error" {
            fmsg = Some(alert_danger( flash.msg() ));
        } else if flash.name() == "warning" {
            fmsg = Some(alert_warning( flash.msg() ));
        } else if flash.name() == "success" {
            fmsg = Some(alert_success( flash.msg() ));
        } else {
            fmsg = Some(alert_info( flash.msg() ));
        }
    }  else {
        fmsg = None;
    }
    fmsg
}

pub fn admin_nav_username(username: &str) -> String {
    format!(r##"
                        <li class="v-nav-item nav-item dropdown">
                            <a class="nav-link dropdown-toggle" href="#" id="navbarDropdown" role="button" data-toggle="dropdown" aria-haspopup="true" aria-expanded="false">
                                {user}
                            </a>
                            <div class="dropdown-menu" aria-labelledby="navbarDropdown">
                                <a class="dropdown-item" href="/insert">New Article</a>
                                <!-- <a class="dropdown-item" href="#">Something else here</a> -->
                                <div class="dropdown-divider"></div>
                                <a class="dropdown-item" href="/logout">Logout</a>
                            </div>
                        </li>
"##, user=username)
}

pub fn admin_nav() -> &'static str {
    r##"
                        <li class="v-nav-item nav-item dropdown">
                            <a class="nav-link dropdown-toggle" href="#" id="navbarDropdown" role="button" data-toggle="dropdown" aria-haspopup="true" aria-expanded="false">
                                {user}
                            </a>
                            <div class="dropdown-menu" aria-labelledby="navbarDropdown">
                                <a class="dropdown-item" href="/insert">New Article</a>
                                <!-- <a class="dropdown-item" href="#">Something else here</a> -->
                                <div class="dropdown-divider"></div>
                                <a class="dropdown-item" href="/logout">Logout</a>
                            </div>
                        </li>
"##
}

pub fn admin_nav_login() -> &'static str {
    r##"<li class="v-nav-item nav-item"><a class="nav-link" href="/admin">Login</a></li>"##
}







pub fn alert_danger(msg: &str) -> String {
    format!(r##"
                        <div class="v-centered-msg alert alert-danger" role="alert">
                            {why}
                        </div>
"##, why=msg)
}
pub fn alert_success(msg: &str) -> String {
    format!(r##"
                        <div class="v-centered-msg alert alert-success" role="alert">
                            {why}
                        </div>
"##, why=msg)
}
pub fn alert_info(msg: &str) -> String {
    format!(r##"
                        <div class="v-centered-msg alert alert-info" role="alert">
                            {why}
                        </div>
"##, why=msg)
}
pub fn alert_warning(msg: &str) -> String {
    format!(r##"
                        <div class="v-centered-msg alert alert-warning" role="alert">
                            {why}
                        </div>
"##, why=msg)
}
pub fn alert_primary(msg: &str) -> String {
    format!(r##"
                        <div class="v-centered-msg alert alert-primary" role="alert">
                            {why}
                        </div>
"##, why=msg)
}


pub fn login_form(url: &str) -> String {
    format!(r##"
                        <form id="needs-validation" action="{url}" name="login_form" method="post" novalidate>
                            <div class="form-group" id="userGroup">
                                <label for="usernameField">Email Address</label>
                                <div class="col-md-9 mb-3">
                                    <input type="text" name="username" value="" class="form-control" id="usernameField" aria-describedby="idHelp" placeholder="Username" required>
                                    <div class="invalid-feedback">
                                        Please specify a username
                                    </div>
                                </div>
                                <!-- <small id="idHelp" class="form-text text-muted">Your email address will not be shared with anyone else.</small> -->
                            </div>
                            <div class="form-group" id="passGroup">
                                <label for="passwordField">Password</label>
                                <div class="col-md-9 mb-3">
                                    <input type="password" name="password" class="form-control" id="passwordField" placeholder="Password" required>
                                    <div class="invalid-feedback">
                                        A password is requierd.
                                    </div>
                                    <input type="password" id="passwordHidden" class="hidden-pass form-control">
                                </div>
                            </div>
                            <div class="v-submit">
                                <button type="submit" class="btn btn-primary" id="submit-button-id">Login</button>
                            </div>
                            <!-- <button type="submit" class="btn btn-faded" id="submit-button-id">Login</button> -->
                            <!-- <button type="submit" class="btn btn-dark" id="submit-button-id">Login</button> -->
                            <!-- <button type="submit" class="btn btn-success" id="submit-button-id">Login</button> -->
                        </form>
"##, url=url)
}

// http://localhost:8000/admin
pub fn login_form_fail(url: &str, user: &str, why: &str) -> String {
    format!(r##"
                        {alert}
                        <form id="needs-validation" action="{url}" name="login_form" method="post" novalidate>
                            <div class="form-group" id="userGroup">
                                <label for="usernameField">Email Address</label>
                                <div class="col-md-9 mb-3">
                                    <input type="text" name="username" value="{user}" class="form-control" id="usernameField" aria-describedby="idHelp" placeholder="Username" required>
                                    <div class="invalid-feedback">
                                        Please specify a username
                                    </div>
                                </div>
                                <!-- <small id="idHelp" class="form-text text-muted">Your email address will not be shared with anyone else.</small> -->
                            </div>
                            <div class="form-group" id="passGroup">
                                <label for="passwordField">Password</label>
                                <div class="col-md-9 mb-3">
                                    <input type="password" name="password" class="form-control" id="passwordField" placeholder="Password" required>
                                    <div class="invalid-feedback">
                                        A password is requierd.
                                    </div>
                                    <input type="password" id="passwordHidden" class="hidden-pass form-control">
                                </div>
                            </div>
                            <div class="v-submit">
                                <button type="submit" class="btn btn-primary" id="submit-button-id">Login</button>
                            </div>
                            <!-- <button type="submit" class="btn btn-faded" id="submit-button-id">Login</button> -->
                            <!-- <button type="submit" class="btn btn-dark" id="submit-button-id">Login</button> -->
                            <!-- <button type="submit" class="btn btn-success" id="submit-button-id">Login</button> -->
                        </form>
"##, url=url, user=user, alert=alert_danger(&format!("Login failed: {}", why)))
}

pub fn link_tags(tags: &Vec<String>) -> String {
    let mut contents = String::new();
    for t in tags {
        contents.push_str(&format!(" <a href=\"{url}tag?tag={tag}\">{tag}</a>", url=BLOG_URL, tag=t));
    }
    contents
}

