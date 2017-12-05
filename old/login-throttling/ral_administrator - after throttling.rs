
use rocket::{Request, Outcome};
// use rocket::request::FromRequest;
use rocket::request::{FromRequest, FromForm, FormItems};
use std::collections::HashMap;
use std::str::{from_utf8};
use chrono::prelude::*;
use chrono::{NaiveDate, NaiveDateTime};


use super::PGCONN;
// use password::*;
use rocket_auth_login::authorization::*;
use rocket_auth_login::sanitization::*;
// use auth::sanitization::*;

const MAX_ATTEMPTS: i16 = 8;
const LOCKOUT_DURATION: u32 = 6; // 900 seconds = 15 minutes


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdministratorCookie {
    pub userid: u32,
    pub username: String,
    pub display: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdministratorForm {
    pub username: String,
    pub password: String,
}

impl CookieId for AdministratorCookie {
    fn cookie_id<'a>() -> &'a str {
        "ascid"
    }
}

impl CookieId for AdministratorForm {
    fn cookie_id<'a>() -> &'a str {
        "ascid"
    }
} 

impl AuthorizeCookie for AdministratorCookie {
    fn store_cookie(&self) -> String {
        ::serde_json::to_string(self).expect("Could not serialize")
    }
    
    
    #[allow(unused_variables)]
    fn retrieve_cookie(string: String) -> Option<Self> {
        let mut des_buf = string.clone();
        let des: Result<AdministratorCookie, _> = ::serde_json::from_str(&mut des_buf);
        if let Ok(cooky) = des {
            Some(cooky)
        } else {
            None
        }
    }
}

impl AuthorizeForm for AdministratorForm {
    type CookieType = AdministratorCookie;
    
    fn authenticate(&self) -> Result<Self::CookieType, AuthFail> {
        let conn = PGCONN.lock().unwrap();
        
        // Could make two queries: authstr and failstr
        // authstr is same as is, gets user data
        // failstr returns the username, is_admin, a bool indicating whether password was correct,
        //     and the attempts and lockout status/date
        
        let authstr = format!(r#"
            SELECT u.userid, u.username, u.display FROM users u WHERE u.username = '{username}' AND is_admin = true AND 
                u.hash_salt = crypt('{password}', u.hash_salt) AND lockout IS NULL"#, username=&self.username, password=&self.password);
        
        let lockout_qrystr = format!("SELECT u.username, u.attempts, u.lockout, LOCALTIMESTAMP as now, crypt('{pass}', u.hash_salt) = u.hash_salt as check FROM users u WHERE u.username = '{user}' AND u.lockout IS NOT NULL", user=&self.username, pass=&self.password);
        
        // let is_user_qrystr = format!("SELECT username, attempts, lockout, LOCALTIMESTAMP as now, LOCALTIMESTAMP + interval '{lock_duration} seconds' as lock_duration  FROM users WHERE username = '{user}'", user=&self.username, lock_duration=LOCKOUT_DURATION);
        let is_user_qrystr = format!("SELECT username, attempts, lockout FROM users WHERE username = '{}'", &self.username);
        let is_admin_qrystr = format!("SELECT userid FROM users WHERE username = '{}' AND is_admin = '1'", &self.username);
        let password_qrystr = format!("SELECT username, attempts FROM users WHERE username = '{}' AND hash_salt = crypt('{}', hash_salt)", &self.username, &self.password);
        
        println!("Running: {}", authstr);
        
        // Checking to see if user credentials are valid
        // does not work if user is locked out
        if let Ok(qry) = conn.query(&authstr, &[]) {
            if !qry.is_empty() && qry.len() == 1 {
                let row = qry.get(0);
                
                let display_opt = row.get_opt(2);
                let display = match display_opt {
                    Some(Ok(d)) => Some(d),
                    _ => None,
                };
                
                let reset_attempts = format!("UPDATE users SET attempts = 0 WHERE username = '{}'", &self.username);
                conn.query(&reset_attempts, &[]);
                
                return Ok(AdministratorCookie {
                    userid: row.get(0),
                    username: row.get(1),
                    display,
                });
            }
        }
        
        // Check if the user is locked out
        if let Ok(eqry) = conn.query(&lockout_qrystr, &[]) {
            if !eqry.is_empty() && eqry.len() != 0 {
                println!("User has been locked out!!  Query ran: {}", lockout_qrystr);
                let row = eqry.get(0);
                let username: String = row.get(0);
                let attempts: i16 = row.get(1);
                let lockout_opt: Option<NaiveDateTime> = row.get(2);
                let lockout = lockout_opt.expect("Error unwrapping lockout value");
                let now: NaiveDateTime = row.get(3);
                let valid: bool = row.get(4);
                
                // let now = Local::now().naive_local();
                
                // if the lockout has expired unlock the account but do not reset the attempts
                if now > lockout {
                    // do not increment attempt it will be incremented when calling authenticate() again
                    println!("Lockout has expired, valid: {}", valid);
                    
                    if valid {
                        let unlock_qrystr = format!("UPDATE users SET lockout = NULL, attempts = 0 WHERE username = '{}' RETURNING userid, username, display", &self.username);
                        println!("Lockout has expired, credentials valid, running: {}", unlock_qrystr);
                        if let Ok(aqry) = conn.query(&unlock_qrystr, &[]) {
                            if !aqry.is_empty() && aqry.len() == 1 {
                                let row = aqry.get(0);
                                let display_opt = row.get_opt(2);
                                let display = match display_opt {
                                    Some(Ok(d)) => Some(d),
                                    _ => None,
                                };
                                return Ok(AdministratorCookie {
                                    userid: row.get(0),
                                    username: row.get(1),
                                    display,
                                });
                            } else {
                                return Err(AuthFail::new(self.username.clone(), "Account unlocked, unknown error occurred 1.".to_string()));
                            }
                        } else {
                            return Err(AuthFail::new(self.username.clone(), "Account unlocked, unknown error occurred 2.".to_string()));
                        }
                    } else {
                        let unlock_qrystr = format!("UPDATE users SET lockout = NULL WHERE username = '{}'", &self.username);
                        
                        println!("Lockout has expired. Running: {}", unlock_qrystr);
                        conn.query(&unlock_qrystr, &[]);
                        
                        return Err(AuthFail::new(self.username.clone(), "Account Unlocked.  Invalid username / password combination.".to_string()));
                    }
                    
                    // return self.authenticate();
                } else {
                    println!("User account is still locked!");
                    return Err( AuthFail::new(self.username.clone(), "User has been locked due to excessive login attempts.  Please try again later.".to_string()) );
                }
                
            }
        }
        
        // let mut user_lock: Option<NaiveDateTime> = Local::now().naive_local();;
        
        let mut attempts: i16 = 0;
        
        // Check if the specified username is an actual user account
        // if not return an error
        // otherwise get the number of attempts that have been made to login to the account
        if let Ok(eqry) = conn.query(&is_user_qrystr, &[]) {
            if eqry.is_empty() || eqry.len() == 0 {
                return Err(AuthFail::new(self.username.clone(), "Username was not found.".to_string()));
            } else {
                let row = eqry.get(0);
                attempts = row.get(1);
            }
        }
        
        attempts += 1;
        println!("{} have made {} attempts to login.", &self.username, attempts);
        // Check the remainder of attempts divided by MAX_ATTEMPTS
        // If the result is 0 that means they have already tried the maximum number of attempts
        //   before the user account is locked
        // The remainder is used so the total attempts can be tracked without having to reset it
        //   after each lockout has ended
        let attempt_qrystr = if attempts % MAX_ATTEMPTS == 0 {
            let inc_qrystr = format!("UPDATE users SET attempts = attempts+1, lockout = LOCALTIMESTAMP + interval '{lockout}' WHERE username = '{user}'", user=&self.username, lockout=LOCKOUT_DURATION);
            println!("Running query to lockout the user and increment attempts: {}", &inc_qrystr);
            conn.query(&inc_qrystr, &[]);
        } else {
            let inc_qrystr = format!("UPDATE users SET attempts = attempts+1 WHERE username = '{}'", &self.username);
            println!("Running query to increment attempts: {}", &inc_qrystr);
            conn.query(&inc_qrystr, &[]);
        };
        
        
        // if adding one to the attempts made it evenly divide into MAX_ATTEMPTS
        // then it means the user account should be locked
        // if attempts % MAX_ATTEMPTS == 0 {
        //     let lock 
        // }
        // // recent tries
        // let tries = attempts % MAX_ATTEMPTS;
        
        
        
        // Check if the user is an administrator
        if let Ok(eqry) = conn.query(&is_admin_qrystr, &[]) {
            if eqry.is_empty() || eqry.len() == 0 {
                // In production this message may be more harmful than useful as it
                // would be able to tell anyone who is an administrator and thus the
                // message should be changed to something like Unkown error or Invalid username/password
                return Err(AuthFail::new(self.username.clone(), "User does not have administrator priveleges.".to_string()));
            }
        }
        // Check if the password is correct
        if let Ok(eqry) = conn.query(&password_qrystr, &[]) {
            if eqry.is_empty() || eqry.len() == 0 {
                return Err(AuthFail::new(self.username.clone(), "Invalid username / password combination.".to_string()));
            }
        }
        Err(AuthFail::new(self.username.clone(), "Unknown error..".to_string()))
    }
    
    fn new_form(user: &str, pass: &str, _extras: Option<HashMap<String, String>>) -> Self {
        AdministratorForm {
            username: user.to_string(),
            password: pass.to_string(),
        }
    }
    
}

impl<'a, 'r> FromRequest<'a, 'r> for AdministratorCookie {
    type Error = ();
    
    fn from_request(request: &'a Request<'r>) -> ::rocket::request::Outcome<AdministratorCookie,Self::Error>{
        let cid = AdministratorCookie::cookie_id();
        let mut cookies = request.cookies();
        
        match cookies.get_private(cid) {
            Some(cookie) => {
                if let Some(cookie_deserialized) = AdministratorCookie::retrieve_cookie(cookie.value().to_string()) {
                    Outcome::Success(
                        cookie_deserialized
                    )
                } else {
                    Outcome::Forward(())
                }
            },
            None => Outcome::Forward(())
        }
    }
}




// impl<'f> FromForm<'f> for AdministratorForm {
//     type Error = &'static str;
    
//     fn from_form(form_items: &mut FormItems<'f>, _strict: bool) -> Result<Self, Self::Error> {
//         // let mut user_pass = HashMap::new();
//         let mut user: String = String::new();
//         let mut pass: String = String::new();
//         // let mut pass: Vec<u8> = Vec::new();
//         let mut extras: HashMap<String, String> = HashMap::new();
        
//         for (key,value) in form_items {
//             match key.as_str(){
//                 "username" => {
//                     // user = sanitize(&value.url_decode().unwrap_or(String::new()));
//                     user = AdministratorForm::clean_username(&value.url_decode().unwrap_or(String::new()));
//                 },
//                 "password" => {
//                     // pass = sanitize_password(&value.url_decode().unwrap_or(String::new()));
//                     pass = AdministratorForm::clean_password(&value.url_decode().unwrap_or(String::new()));
//                     // pass = value.bytes().collect();
//                 },
//                 // _ => {},
//                 a => {
//                     // extras.insert( a.to_string(), sanitize( &value.url_decode().unwrap_or(String::new()) ) );
//                     extras.insert( a.to_string(), AdministratorForm::clean_extras( &value.url_decode().unwrap_or(String::new()) ) );
//                 },
//             }
//         }
        
//         // println!("Creating login form data structure with:\nUser: {}\nPass: {}\nExtras: {:?}", user, pass, extras);
        
//         // Do not need to check for username / password here,
//         // if the authentication method requires them it will
//         // fail at that point.
//         Ok(
//             if extras.len() == 0 {
//               AdministratorForm::new_form(&user, &pass, None)
//            } else {
//                AdministratorForm::new_form(&user, &pass, Some(extras))
//            }
//         )
//     }
// }


// impl<'f> FromForm<'f> for LoginCont<AdministratorForm> {
//     type Error = &'static str;
//     
//     fn from_form(form_items: &mut FormItems<'f>, _strict: bool) -> Result<Self, Self::Error> {
//         // let mut user_pass = HashMap::new();
//         let mut user: String = String::new();
//         let mut pass: String = String::new();
//         // let mut pass: Vec<u8> = Vec::new();
//         let mut extras: HashMap<String, String> = HashMap::new();
//         
//         for (key,value) in form_items {
//             match key.as_str(){
//                 "username" => {
//                     // user = sanitize(&value.url_decode().unwrap_or(String::new()));
//                     user = AdministratorForm::clean_username(&value.url_decode().unwrap_or(String::new()));
//                 },
//                 "password" => {
//                     // pass = sanitize_password(&value.url_decode().unwrap_or(String::new()));
//                     pass = AdministratorForm::clean_password(&value.url_decode().unwrap_or(String::new()));
//                     // pass = value.bytes().collect();
//                 },
//                 // _ => {},
//                 a => {
//                     // extras.insert( a.to_string(), sanitize( &value.url_decode().unwrap_or(String::new()) ) );
//                     extras.insert( a.to_string(), AdministratorForm::clean_extras( &value.url_decode().unwrap_or(String::new()) ) );
//                 },
//             }
//         }
//         
//         // println!("Creating login form data structure with:\nUser: {}\nPass: {}\nExtras: {:?}", user, pass, extras);
//         
//         // Do not need to check for username / password here,
//         // if the authentication method requires them it will
//         // fail at that point.
//         Ok(
//             LoginCont {
//                 form: if extras.len() == 0 {
//                           AdministratorForm::new_form(&user, &pass, None)
//                        } else {
//                            AdministratorForm::new_form(&user, &pass, Some(extras))
//                        },
//             }
//         )
//     }
// }






