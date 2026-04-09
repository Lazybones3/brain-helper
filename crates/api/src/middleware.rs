use std::time::Duration;

use chrono::Utc;
use chrono_tz::US::Eastern;
use http::{Extensions, StatusCode, header::RETRY_AFTER};
use reqwest::{Request, Response};
use reqwest_middleware::{Middleware, Next, Result};
use tracing::{debug, info, warn};

use crate::{brain_api, session};

pub struct LoginMiddleware;

#[async_trait::async_trait]
impl Middleware for LoginMiddleware {
    async fn handle(
        &self,
        req: Request,
        extensions: &mut Extensions,
        next: Next<'_>,
    ) -> Result<Response> {
        let req_url = req.url().path();
        if req_url != "/authentication" {
            let client = session::global();
            if client.is_expired().await {
                info!("{} has expired", req_url);
                brain_api::start_session().await?;
            } else {
                info!("{} is not expired", req_url);
            }
        }
        next.run(req, extensions).await
    }
}

pub struct RequestLimitMiddleware;

#[async_trait::async_trait]
impl Middleware for RequestLimitMiddleware {
    async fn handle(
        &self,
        req: Request,
        extensions: &mut Extensions,
        next: Next<'_>,
    ) -> Result<Response> {
        let res = next.run(req.try_clone().unwrap(), extensions).await?;

        // 1. Handle Retry-After Header (Standard)
        if let Some(retry_after) = res.headers().get("retry-after") {
            let seconds = retry_after
                .to_str()
                .unwrap_or("0")
                .parse::<f64>()
                .unwrap_or(0.0);
            if seconds > 0.0 {
                info!("retry-after: {}", seconds);
                tokio::time::sleep(Duration::from_secs_f64(seconds + 1.)).await;
                return Ok(res);
            }
        }

        let status = res.status();
        info!("{} status: {} headers: {:?}", req.url().path(), status, res.headers());
        let mut builder = http::Response::builder().status(status);
        if let Some(headers_mut) = builder.headers_mut() {
            *headers_mut = res.headers().clone();
        }
        let bytes = res.bytes().await?;

        // 2. Handle Platform-Specific 429 Errors
        if StatusCode::TOO_MANY_REQUESTS.eq(&status) {
            let body: serde_json::Value = serde_json::from_slice(&bytes).unwrap_or_default();
            let detail = body["detail"].as_str().unwrap_or_default();

            match detail {
                "DAILY_SIMULATION_LIMIT_EXCEEDED" => {
                    let now_est = Utc::now().with_timezone(&Eastern);
                    let next_day = (now_est + Duration::from_hours(24))
                        .date_naive()
                        .and_hms_opt(0, 0, 0)
                        .unwrap();
                    let wait_secs = (next_day - now_est.naive_local()).num_seconds() as u64;

                    warn!(
                        "Daily limit reached. Waiting until EST midnight ({}s)...",
                        wait_secs + 100
                    );
                    tokio::time::sleep(std::time::Duration::from_secs(wait_secs + 100)).await;
                }
                "CONCURRENT_SIMULATION_LIMIT_EXCEEDED" => {
                    tokio::time::sleep(std::time::Duration::from_secs(10)).await;
                }
                _ => {}
            }
        }

        let new_res = reqwest::Response::from(builder.body(bytes).unwrap());
        Ok(new_res)
    }
}
