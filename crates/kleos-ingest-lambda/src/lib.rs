use aws_lambda_events::event::apigw::ApiGatewayProxyRequest;
use derive_more::{Display, From};
use kleos_lib::{Event, Payload};
use serde::{Deserialize, Serialize};
use std::convert::TryFrom;
use thiserror::Error;
use uuid::Uuid;

// --- Modules ---

pub mod config;
pub mod ingestor;
pub mod publisher;

// --- Errors ---

#[derive(Debug, Error)]
pub enum DomainError {
    #[error("Invalid amount: {0}")]
    InvalidAmount(f64),
    #[error("Invalid page URL: {0}")]
    InvalidPageUrl(String),
    #[error("Missing request body")]
    MissingBody,
    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),
}

// --- Simple Types (Single Case Unions) ---

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Display, From)]
pub struct UserId(Uuid);

impl UserId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
}

impl Default for UserId {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Display, From)]
pub struct ProductId(Uuid);

impl ProductId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
}

impl Default for ProductId {
    fn default() -> Self {
        Self::new()
    }
}

// --- Constrained Types ---

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Display)]
pub struct PageUrl(String);

impl PageUrl {
    pub fn create(url: String) -> Result<Self, DomainError> {
        if url.is_empty() {
            return Err(DomainError::InvalidPageUrl(
                "URL cannot be empty".to_string(),
            ));
        }
        // In a real app, we'd do more robust validation here
        if !url.starts_with("http") && !url.starts_with("/") {
            return Err(DomainError::InvalidPageUrl(format!(
                "URL must start with http or /: {}",
                url
            )));
        }
        Ok(Self(url))
    }

    pub fn value(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, Display)]
pub struct Amount(f64);

impl Amount {
    pub fn create(amount: f64) -> Result<Self, DomainError> {
        if amount < 0.0 {
            return Err(DomainError::InvalidAmount(amount));
        }
        Ok(Self(amount))
    }

    pub fn value(&self) -> f64 {
        self.0
    }
}

// --- Choice Types (Domain Events) ---

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "data")]
pub enum UserAction {
    PageView {
        user_id: UserId,
        url: PageUrl,
    },
    Click {
        user_id: UserId,
        element_id: String,
        url: PageUrl,
    },
    Purchase {
        user_id: UserId,
        product_id: ProductId,
        amount: Amount,
    },
}

// --- Constructor Helpers & Logic ---

impl UserAction {
    pub fn page_view(user_id: UserId, url: PageUrl) -> Self {
        Self::PageView { user_id, url }
    }

    pub fn click(user_id: UserId, element_id: String, url: PageUrl) -> Self {
        Self::Click {
            user_id,
            element_id,
            url,
        }
    }

    pub fn purchase(user_id: UserId, product_id: ProductId, amount: Amount) -> Self {
        Self::Purchase {
            user_id,
            product_id,
            amount,
        }
    }

    pub fn try_from_json(json: &str) -> Result<Self, DomainError> {
        serde_json::from_str(json).map_err(DomainError::SerializationError)
    }

    pub fn into_event(self) -> Event<UserAction> {
        Event::new(Payload::new(self))
    }
}

/// Extract and validate UserAction from API Gateway request body.
///
/// This provides clean, type-safe extraction of domain events from HTTP requests.
impl TryFrom<ApiGatewayProxyRequest> for UserAction {
    type Error = DomainError;

    fn try_from(request: ApiGatewayProxyRequest) -> Result<Self, Self::Error> {
        // Extract body from request
        let body = request.body.ok_or(DomainError::MissingBody)?;

        // Parse and validate
        UserAction::try_from_json(&body)
    }
}
