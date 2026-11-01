//! Role: Form state and the context shared by the components.

use crate::http::{Request, Response};
use crate::storage::HistoryEntry;
use kivact::prelude::*;
use std::sync::Arc;

pub(crate) const METHODS: [&str; 2] = ["GET", "POST"];

#[derive(Clone, Debug, PartialEq)]
pub struct FormState {
    pub method: String,
    pub url: String,
    pub headers: String,
    pub body: String,
}

impl Default for FormState {
    fn default() -> Self {
        Self {
            method: "GET".into(),
            url: "http://127.0.0.1:8000/".into(),
            headers: "Accept: application/json".into(),
            body: String::new(),
        }
    }
}

impl FormState {
    pub fn from_request(request: &Request) -> Self {
        Self {
            method: request.method.clone(),
            url: request.url.clone(),
            headers: request.headers.clone(),
            body: request.body.clone(),
        }
    }

    pub fn to_request(&self) -> Request {
        Request {
            method: self.method.clone(),
            url: self.url.clone(),
            headers: self.headers.clone(),
            body: self.body.clone(),
        }
    }
}

/// Copy handles provided by the root (`context::<Postman>()`).
#[derive(Clone, Copy)]
pub(crate) struct Postman {
    pub form: Signal<FormState>,
    pub history: Signal<Vec<HistoryEntry>>,
    /// Messages outside the request lifecycle (validation, copy, focus, storage).
    pub notice: Signal<Option<String>>,
    /// The latest request wins; its state is the single source of pending/result.
    pub send: Action<Arc<Response>, Request>,
}
