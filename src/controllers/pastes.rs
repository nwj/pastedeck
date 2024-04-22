use crate::{
    db::Database,
    models::paste::Paste,
    views::pastes::{IndexPastesTemplate, NewPastesTemplate, ShowPastesTemplate},
};
use axum::{
    extract::{Form, Path, State},
    http::{header::HeaderMap, StatusCode},
    response::{IntoResponse, Redirect, Response},
};
use serde::Deserialize;

pub async fn index(State(db): State<Database>) -> IndexPastesTemplate {
    let pastes = Paste::all(&db).await.unwrap();
    IndexPastesTemplate { pastes }
}

pub async fn new() -> NewPastesTemplate {
    NewPastesTemplate {}
}

#[derive(Deserialize, Debug)]
pub struct CreateFormInput {
    pub text: String,
}

pub async fn create(State(db): State<Database>, Form(input): Form<CreateFormInput>) -> Response {
    Paste::insert(&db, input.text).await.unwrap();
    Redirect::to("/pastes").into_response()
}

pub async fn show(Path(id): Path<i64>, State(db): State<Database>) -> impl IntoResponse {
    let maybe_paste = Paste::find(&db, id).await.unwrap();
    let status_code = if maybe_paste.is_some() {
        StatusCode::OK
    } else {
        StatusCode::NOT_FOUND
    };
    (status_code, ShowPastesTemplate { maybe_paste })
}

pub async fn destroy(Path(id): Path<i64>, State(db): State<Database>) -> impl IntoResponse {
    Paste::delete(&db, id).await.unwrap();
    let mut headers = HeaderMap::new();
    headers.insert("HX-Redirect", "/pastes".parse().unwrap());
    headers
}
