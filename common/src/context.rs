use std::sync::Arc;

use serde::Serialize;
use type_map::concurrent::TypeMap;

use crate::repository::Repository;

pub struct InnerContext {
    pub repositories: TypeMap,
    pub client: reqwest::Client,
}

pub struct Context(pub Arc<InnerContext>);

impl Clone for Context {
    fn clone(&self) -> Self {
        Self(Arc::clone(&self.0))
    }
}

pub struct ServiceRequest<'a, 'b, T = ()> {
    // TODO: return error, if we try to send body into GET request
    client: &'a reqwest::Client,
    method: reqwest::Method,
    url: Option<String>,
    body: Option<&'b T>,
}

impl<'a, 'b, T: Serialize> ServiceRequest<'a, 'b, T> {
    pub fn new(client: &'a reqwest::Client) -> Self {
        Self {
            client,
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
        let body = self.body.unwrap();
        let mut request = self.client.request(self.method, url);
        if let Some(body) = self.body {
            request = request.json(body);
        }
        let response = request.send().await?;
        Ok(response)
    }
}

impl Context {
    pub fn new() -> Self {
        Self(Arc::new(InnerContext {
            repositories: TypeMap::new(),
            client: reqwest::Client::new(),
        }))
    }

    pub fn get_repository<T: 'static>(&self) -> Option<Repository<T>> {
        self.0.repositories.get::<Repository<T>>().cloned()
    }

    pub fn make_request<T: Serialize>(&self) -> ServiceRequest<T> {
        ServiceRequest::<T>::new(&self.0.client)
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
