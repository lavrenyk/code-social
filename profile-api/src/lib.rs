mod config;
mod model;
mod utils;

use anyhow::Result;
use spin_sdk::{
    http::{IntoResponse, Request, Response, Method},
    http_component,
};
use config::Config;
use utils::{bad_request,internal_server_error,method_not_allowed,not_found, ok, no_content};
use model::Profile;
use crate::Api::MethodNotAllowed;

enum Api {
    Create(Profile),
    ReadByHandle(String),
    Update(Profile),
    Delete(Profile),
    BadRequest,
    NotFound,
    MethodNotAllowed,
}

#[http_component]
fn profile_api(req: Request) -> Result<impl IntoResponse> {
    let config = Config::get();
    
    match api_from_request(req) {
        Api::BadRequest => bad_request(),
        MethodNotAllowed => method_not_allowed(),
        Api::Create(model) => handle_create(&config.db_url, model),
        Api::Update(model) => handle_update(&config.db_url, model),
        Api::ReadByHandle(handle) => handle_read_by_handle(&config.db_url, handle),
        Api::Delete(handle) => handle_delete_by_handle(&config.db_url, handle),
        _ => not_found()
    }
}

fn api_from_request(req: Request) -> Api {
    match *req.method() {
        Method::Post => match Profile::from_bytes(&req.body().as_ref()) {
            Ok(model) => Api::Create(model),
            Err(_) => Api::BadRequest,
        }
        Method::Get => match Profile::from_path(&req.header("spin-path-info")) {
            Ok(model) => Api::ReadByHandle(model.handle),
            Err(_) => {
                Api::NotFound
            },
        },
        Method::Put => match Profile::from_bytes(&req.body().as_ref()) {
            Ok(model) => Api::Update(model),
            Err(_) => Api::BadRequest,
        },
        Method::Delete => match Profile::from_path(&req.header("spin-path-info")) {
            Ok(model) => Api::Delete(model),
            Err(_) => Api::NotFound,
        },
        _ => MethodNotAllowed,
    }
}


fn handle_create(db_url: &str, model: Profile) -> Result<Response> {
    model.insert(db_url)?;
    Ok(Response::builder()
        .status(http::StatusCode::CREATED.as_u16())
        .header(http::header::LOCATION.as_str(), format!("/api/profile/{}", model.handle))
        .build()
    )
}

fn handle_read_by_handle(db_url: &str, handle: String) -> Result<Response> {
    match Profile::get_by_handle(handle.as_str(), &db_url) {
        Ok(model) => ok(serde_json::to_string(&model)?),
        Err(_) => not_found()
    }
}

fn handle_update(db_url: &str, model: Profile) -> Result<Response> {
    model.update(&db_url)?;
    handle_read_by_handle(&db_url, model.handle)
}

fn handle_delete_by_handle(db_url: &str, model: Profile) -> Result<Response> {
    match model.delete(&db_url) {
        Ok(_) => no_content(),
        Err(_) => internal_server_error(String::from("Error while deleting profile"))
    }
}
