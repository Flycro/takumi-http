use std::{collections::HashMap, future::Future};

use axum::{
    body::Bytes,
    extract::{FromRequest, Multipart, Request},
    http::header::CONTENT_TYPE,
};
use serde::de::DeserializeOwned;

use crate::error::ApiError;

pub struct JsonOrMultipart<T> {
    pub data: T,
    pub files: Vec<(String, Bytes)>,
}

impl<S, T> FromRequest<S> for JsonOrMultipart<T>
where
    S: Send + Sync,
    T: DeserializeOwned + MultipartParseable + Send,
{
    type Rejection = ApiError;

    #[allow(clippy::manual_async_fn)]
    fn from_request(
        req: Request,
        state: &S,
    ) -> impl Future<Output = Result<Self, Self::Rejection>> + Send
    where
        Self: Sized,
    {
        async move {
            let content_type = req
                .headers()
                .get(CONTENT_TYPE)
                .and_then(|v| v.to_str().ok())
                .unwrap_or("")
                .to_string();

            if content_type.starts_with("multipart/form-data") {
                let mut multipart = Multipart::from_request(req, state)
                    .await
                    .map_err(|e| ApiError::MultipartError(e.to_string()))?;

                let mut fields: HashMap<String, String> = HashMap::new();
                let mut files: Vec<(String, Bytes)> = Vec::new();

                while let Some(field) = multipart
                    .next_field()
                    .await
                    .map_err(|e| ApiError::MultipartError(e.to_string()))?
                {
                    let name = field.name().unwrap_or("").to_string();

                    if name.starts_with("resource_") || name.starts_with("file_") {
                        let key = name
                            .strip_prefix("resource_")
                            .or_else(|| name.strip_prefix("file_"))
                            .unwrap_or(&name)
                            .to_string();

                        let data = field
                            .bytes()
                            .await
                            .map_err(|e| ApiError::MultipartError(e.to_string()))?;

                        files.push((key, data));
                    } else {
                        let text = field
                            .text()
                            .await
                            .map_err(|e| ApiError::MultipartError(e.to_string()))?;

                        fields.insert(name, text);
                    }
                }

                let data = T::from_multipart_fields(fields, &files)?;

                Ok(JsonOrMultipart { data, files })
            } else {
                let bytes = Bytes::from_request(req, state)
                    .await
                    .map_err(|e| ApiError::BadRequest(e.to_string()))?;

                let data: T = serde_json::from_slice(&bytes).map_err(ApiError::JsonError)?;

                Ok(JsonOrMultipart {
                    data,
                    files: Vec::new(),
                })
            }
        }
    }
}

pub trait MultipartParseable: Sized {
    fn from_multipart_fields(
        fields: HashMap<String, String>,
        files: &[(String, Bytes)],
    ) -> Result<Self, ApiError>;
}
