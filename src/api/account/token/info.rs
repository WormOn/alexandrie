use diesel::prelude::*;
use http::StatusCode;
use serde::{Deserialize, Serialize};
use tide::{Request, Response};

use crate::db::models::AuthorToken;
use crate::db::schema::*;
use crate::utils;
use crate::{Error, State};

/// Request body for this route.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RequestBody {
    /// The registry token to get information about.
    pub token: String,
}

/// Response body for this route.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ResponseBody {
    /// The token name.
    pub name: String,
    /// The expiration date for this token.
    pub expires_at: Option<String>,
}

/// Route to get information about a registry token.
pub async fn get(req: Request<State>) -> Result<Response, Error> {
    let name = req.param::<String>("name").unwrap();

    let state = req.state().clone();
    let repo = &state.repo;

    //? Is the author logged in ?
    let headers = req.headers().clone();
    let author = repo
        .run(move |conn| utils::checks::get_author(conn, &headers))
        .await;
    let author = match author {
        Some(author) => author,
        None => {
            return Ok(utils::response::error(
                StatusCode::UNAUTHORIZED,
                "please log in first to access token information",
            ));
        }
    };

    //? Fetch the token from the database.
    let token = repo
        .run(move |conn| {
            author_tokens::table
                .filter(author_tokens::name.eq(name.as_str()))
                .filter(author_tokens::author_id.eq(author.id))
                .first::<AuthorToken>(conn)
                .optional()
        })
        .await?;

    //? Was a token found ?
    let token = match token {
        Some(token) => token,
        None => {
            return Ok(utils::response::error(
                StatusCode::NOT_FOUND,
                "no token was found for the supplied name",
            ))
        }
    };

    let expires_at = None;
    let response = ResponseBody {
        name: token.name,
        expires_at,
    };
    Ok(utils::response::json(&response))
}

/// Route to get information about a registry token.
pub async fn post(mut req: Request<State>) -> Result<Response, Error> {
    let state = req.state().clone();
    let repo = &state.repo;

    //? Is the author logged in ?
    let headers = req.headers().clone();
    let author = repo
        .run(move |conn| utils::checks::get_author(conn, &headers))
        .await;
    let author = match author {
        Some(author) => author,
        None => {
            return Ok(utils::response::error(
                StatusCode::UNAUTHORIZED,
                "please log in first to access token information",
            ));
        }
    };

    //? Parse request body.
    let body: RequestBody = req.body_json().await?;

    //? Fetch the token from the database.
    let token = repo
        .run(move |conn| {
            author_tokens::table
                .filter(author_tokens::token.eq(body.token.as_str()))
                .filter(author_tokens::author_id.eq(author.id))
                .first::<AuthorToken>(conn)
                .optional()
        })
        .await?;

    //? Was a token found ?
    let token = match token {
        Some(token) => token,
        None => {
            return Ok(utils::response::error(
                StatusCode::FORBIDDEN,
                "unauthorized access to this token",
            ))
        }
    };

    let expires_at = None;
    let response = ResponseBody {
        name: token.name,
        expires_at,
    };
    Ok(utils::response::json(&response))
}