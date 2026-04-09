use std::{ops::Deref, sync::OnceLock};

use chrono::{DateTime, Utc};
use reqwest::Client;
use reqwest_middleware::{ClientBuilder, ClientWithMiddleware};
use tokio::sync::RwLock;

use crate::middleware::{LoginMiddleware, RequestLimitMiddleware};

static INSTANCE: OnceLock<SessionManager> = OnceLock::new();

pub struct SessionManager {
    client: ClientWithMiddleware,
    expiry_time: RwLock<Option<DateTime<Utc>>>,
}

impl SessionManager {
    pub async fn set_expiry_time(&self, expiry: DateTime<Utc>) {
        *self.expiry_time.write().await = Some(expiry);
    }

    pub async fn is_expired(&self) -> bool {
        self.expiry_time
            .read()
            .await
            .map_or(true, |expiry| Utc::now() >= expiry)
    }
}

impl Deref for  SessionManager {
    type Target = ClientWithMiddleware;

    fn deref(&self) -> &Self::Target {
        &self.client
    }
}

pub fn global() -> &'static SessionManager {
    INSTANCE.get_or_init(|| {
        let reqwest_client = Client::builder()
            .cookie_store(true)
            .build()
            .expect("Failed to create client");
        let client = ClientBuilder::new(reqwest_client)
            .with(LoginMiddleware)
            .with(RequestLimitMiddleware)
            .build();
        SessionManager { client, expiry_time: RwLock::new(None) }
    })
}
