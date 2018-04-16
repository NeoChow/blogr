use super::*;

#[get("/content/<uri..>")]
pub fn static_pages(start: GenTimer, 
                    uri: PathBuf, 
                    admin: Option<AdministratorCookie>, 
                    user: Option<UserCookie>, 
                    encoding: AcceptCompression, 
                    uhits: UniqueHits, 
                    context: State<ContentContext>, 
                   ) -> Result<ContentRequest, Express> {
    // Could also prevent hotlinking by checking the referrer
    //   and sending an error for referring sites other than BASE or blank
    // Look for the uri in the context, if it exists then make a ContextRequest
    //   which will be passed as the output
    //   before passing ContextRequest as the output, check for admin/user in the context
    //     if the context has user or admin set to true then make sure the admin/user var is_some()
    //   if it does not exist then return an Express instance with an error message
    //   use hbs_template's General template
    // Could also move context out of the ContentReuqest and in the Responder use
    //      let cache = req.guard::<State<HitCount>>().unwrap();
    
    let page = uri.to_string_lossy().into_owned();
    
    if let Ok(ctx_reader) = context.pages.read() {
        if let Some(ctx) = ctx_reader.get(&page) {
            // Permissions check
            if (ctx.admin && admin.is_none()) || (ctx.user && user.is_none()) {
                let template = hbs_template(TemplateBody::General(alert_danger("You do not have sufficient privileges to view this content.")), None, Some("Insufficient Privileges".to_string()), String::from("/error403"), admin, user, None, Some(start.0));
                let express: Express = template.into();
                return Err(express.compress(encoding));
            }
            
            // Build a ContentRequest with the requested files
            let conreq: ContentRequest = ContentRequest {
                encoding,
                route: page,
                start,
            };
            Ok(conreq)
        } else {
            let template = hbs_template(TemplateBody::General(alert_danger("The requested content could not be found.")), None, Some("Content not found.".to_string()), String::from("/error404"), admin, user, None, Some(start.0));
            let express: Express = template.into();
            Err(express.compress(encoding))
        }
    } else {
        let template = hbs_template(TemplateBody::General(alert_danger("An error occurred attempting to access content.")), None, Some("Content not available.".to_string()), String::from("/error404"), admin, user, None, Some(start.0));
        let express: Express = template.into();
        Err(express.compress(encoding))
    }
}

fn err_file_name(name: &str) -> PathBuf {
    if let Ok(mut dir) = env::current_exe() {
        dir.pop();
        // println!("Climbing directory tree into: {}", &dir.display());
        dir.pop();
        // println!("Loading into directory: {}", &dir.display());
        if cfg!(target_os = "windows") {
            dir.set_file_name(&format!("logs\\{}", name));
        } else {
            dir.set_file_name(&format!("logs/{}", name));
        }
        // println!("Load file is: {}", &dir.display());
        dir
    } else {
        PathBuf::from(name)
    }
}

#[get("/download/<uri..>")]
pub fn code_download(start: GenTimer, 
                    uri: PathBuf, 
                    admin: Option<AdministratorCookie>, 
                    user: Option<UserCookie>, 
                    encoding: AcceptCompression, 
                    uhits: UniqueHits, 
                    context: State<ContentContext>, 
                    // cache_lock: State<ContentCacheLock>
                   ) -> Express {
    // If the requested URI cannot be found in the static page cache
    //   maybe try looking in the uploads folder
    
    let page = uri.to_string_lossy().into_owned();
    
    if let Ok(ctx_reader) = context.pages.read() {
            
        if let Some(ctx) = ctx_reader.get(&page) {
            // Permissions check
            if (ctx.admin && admin.is_none()) || (ctx.user && user.is_none()) {
                let template = hbs_template(TemplateBody::General(alert_danger("You do not have sufficient privileges to view this content.")), None, Some("Insufficient Privileges".to_string()), String::from("/error403"), admin, user, None, Some(start.0));
                let express: Express = template.into();
                return express.compress(encoding);
            }
            
            let express: Express = ctx.body.clone().into();
            
            let attachment = ContentDisposition {
                disposition: DispositionType::Attachment,
                parameters: vec![DispositionParam::Filename(
                  Charset::Iso_8859_1, // The character set for the bytes of the filename
                  None, // The optional language tag (see `language-tag` crate)
                  ctx.uri.clone().into_bytes()
                )]
            };
            express
                // Disable cache headers; IE breaks if downloading a file over HTTPS with cache-control headers
                .set_ttl(-2)
                .add_header(attachment)
        } else {
            for log in DOWNLOADABLE_LOGS {
                if &page == log {
                    if admin.is_some() {
                        let err_path = err_file_name(log);
                        println!("Attempting to open {}", err_path.display());
                        if !err_path.exists() {
                            let template = hbs_template(TemplateBody::General(alert_danger("Error log could not be found.")), None, Some("Content not Found".to_string()), String::from("/error404"), admin, user, None, Some(start.0));
                            let express: Express = template.into();
                            return express.compress(encoding);
                        }
                        if let Ok(mut f) = File::open( err_path ) {
                            let mut buffer = Vec::new();
                            f.read_to_end(&mut buffer);
                            let express: Express = buffer.into();
                            
                            let attachment = ContentDisposition {
                                disposition: DispositionType::Attachment,
                                parameters: vec![DispositionParam::Filename(
                                  Charset::Iso_8859_1, // The character set for the bytes of the filename
                                  None, // The optional language tag (see `language-tag` crate)
                                  log.to_string().into_bytes()
                                )]
                            };
                            
                            return express
                                .set_ttl(-2)
                                .add_header(attachment);
                        } else {
                            let template = hbs_template(TemplateBody::General(alert_danger("Error log could not be found.")), None, Some("Content not Found".to_string()), String::from("/error404"), admin, user, None, Some(start.0));
                            let express: Express = template.into();
                            return express.compress(encoding);
                        }
                    } else {
                        let template = hbs_template(TemplateBody::General(alert_danger("You do not have sufficient privileges to view this content.")), None, Some("Insufficient Privileges".to_string()), String::from("/error403"), admin, user, None, Some(start.0));
                        let express: Express = template.into();
                        return express.compress(encoding);
                    }
                }
            }
            let template = hbs_template(TemplateBody::General(alert_danger("The requested download could not be found.")), None, Some("Content not found.".to_string()), String::from("/error404"), admin, user, None, Some(start.0));
            let express: Express = template.into();
            express.compress(encoding)
        }
    } else {
        let template = hbs_template(TemplateBody::General(alert_danger("An error occurred attempting to access content.")), None, Some("Content not available.".to_string()), String::from("/error404"), admin, user, None, Some(start.0));
        let express: Express = template.into();
        express.compress(encoding)
    }
    
}

