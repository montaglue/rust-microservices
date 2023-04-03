use std::sync::Arc;

use anyhow::bail;
use axum::{async_trait, extract::FromRequestParts, http::request::Parts};
use serde::Serialize;
use type_map::concurrent::TypeMap;

use crate::{auth::Auth, error::ServiceError, repository::Repository};

pub struct ServiceState {
    pub repositories: TypeMap,
    pub client: reqwest::Client,
    pub auth: Auth,
}

pub struct HandlerContext {
    pub user_auth: Option<Auth>,
}

pub struct Context(pub Arc<ServiceState>, pub HandlerContext);

pub struct ContextExtractor(pub Context);

pub fn extract_token(parts: &Parts) -> anyhow::Result<Option<&str>> {
    let Some(header) = parts.headers.get("Authorization") else {
        return Ok(None)
    };

    let Some(token) = header.to_str()?.strip_prefix("Bearer ") else {
        bail!("wrong token format")
    };

    Ok(Some(token))
}

#[async_trait]
impl FromRequestParts<Arc<ServiceState>> for ContextExtractor {
    type Rejection = ServiceError;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &Arc<ServiceState>,
    ) -> Result<Self, Self::Rejection> {
        let mut user_auth = None;

        if let Some(token) = extract_token(parts)? {
            user_auth = Some(Auth::from_token(token)?);
        }

        Ok(ContextExtractor(Context(
            Arc::clone(state),
            HandlerContext { user_auth },
        )))
    }
}

pub struct ServiceRequest<'a, 'b, T = ()> {
    // TODO: return error, if we try to send body into GET request
    client: &'a reqwest::Client,
    method: reqwest::Method,
    url: Option<String>,
    body: Option<&'b T>,
    auth: Auth,
}

impl<'a, 'b, T: Serialize> ServiceRequest<'a, 'b, T> {
    pub fn new(client: &'a reqwest::Client, auth: Auth) -> Self {
        Self {
            client,
            auth,
            method: reqwest::Method::GET,
            url: None,
            body: None,
        }
    }

    pub fn get(mut self, url: String) -> Self {
        self.url = Some(url);
        self
    }

    pub fn post(mut self, url: String) -> Self {
        self.url = Some(url);
        self.method = reqwest::Method::POST;
        self
    }

    pub fn patch(mut self, url: String) -> Self {
        self.url = Some(url);
        self.method = reqwest::Method::PATCH;
        self
    }

    pub fn delete(mut self, url: String) -> Self {
        self.url = Some(url);
        self.method = reqwest::Method::DELETE;
        self
    }

    pub fn json(mut self, body: &'b T) -> Self {
        self.body = Some(body);
        self
    }

    pub async fn send(self) -> anyhow::Result<reqwest::Response> {
        let url = self.url.as_ref().unwrap();
        let mut request = self.client.request(self.method, url);
        if let Some(body) = self.body {
            request = request.json(body);
        }
        let response = request.send().await?;
        Ok(response)
    }

    pub fn auth(mut self, auth: Auth) -> Self {
        self.auth = auth;
        self
    }
}

impl Context {
    pub fn get_repository<T: 'static>(&self) -> Option<Repository<T>> {
        self.0.repositories.get::<Repository<T>>().cloned()
    }

    pub fn make_request<T: Serialize>(&self) -> ServiceRequest<T> {
        ServiceRequest::<T>::new(&self.0.client, self.0.auth.clone())
    }
}

pub struct MutationContext<'a> {
    pub context: &'a Context,
    pub current_field: Option<String>,
}

impl<'a> MutationContext<'a> {
    pub fn new(context: &'a Context) -> Self {
        Self {
            current_field: None,
            context,
        }
    }
}
