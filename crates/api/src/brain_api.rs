use std::{io::Write, pin::Pin, time::Duration};

use chrono::Utc;
use futures::StreamExt;
use http::header::LOCATION;
use indicatif::{ProgressBar, ProgressStyle};
use serde::Serialize;
use serde_json::json;
use tracing::{debug, error, info, warn};

use crate::{
    constant::BRAIN_API_URL,
    objects::Credentials,
    parameter::{AlphaConfig, Settings},
    response::{
        ResultsResponse, auth::AuthResponse, datafield::DataField, dataset::Dataset, operator::Operator, simulation::SimulationResult
    },
    session, utils,
};

pub async fn start_session() -> anyhow::Result<()> {
    let client = session::global();
    let creds = get_credentials()?;
    let url = format!("{}/authentication", BRAIN_API_URL);
    let res = client
        .post(url)
        .basic_auth(creds.email, Some(creds.password))
        .send()
        .await?;

    let status = res.status();
    let headers = res.headers().clone();

    if status == reqwest::StatusCode::UNAUTHORIZED {
        if let Some(loc) = headers.get("Location") {
            let personal_url = loc.to_str().unwrap();
            println!("Complete biometrics authentication at: {}", personal_url);
            println!("Press Enter once finished...");
            let mut dummy = String::new();
            std::io::stdin().read_line(&mut dummy).ok();

            // Verification loop
            loop {
                let check = client.post(personal_url).send().await?;
                if check.status() == reqwest::StatusCode::CREATED {
                    break;
                }
                println!("Biometrics not complete. Try again and press Enter.");
                std::io::stdin().read_line(&mut dummy).ok();
            }
        } else {
            eprintln!("Incorrect credentials. Clearing cache...");
            let home = std::env::home_dir().unwrap();
            std::fs::remove_file(home.join("secrets/platform-brain.json")).ok();
        }
    } else {
        let body: AuthResponse = res.json().await?;
        tracing::debug!(
            "New session created (ID: {:p}) with authentication response: {}, {:?}",
            client,
            status,
            body
        );
        let expiry_secs = (body.token.expiry as u64) - 600;
        let expiry_time = Utc::now() + Duration::from_secs(expiry_secs);
        client.set_expiry_time(expiry_time).await;
    }
    Ok(())
}

fn get_credentials() -> anyhow::Result<Credentials> {
    let home = std::env::home_dir().expect("Could not find home directory");
    let secrets_path = home.join("secrets").join("platform-brain.json");

    // Try reading from file first
    if let Ok(content) = std::fs::read_to_string(&secrets_path) {
        if let Ok(creds) = serde_json::from_str::<Credentials>(&content) {
            return Ok(creds);
        }
    }

    // Fallback to env or manual input
    let email = std::env::var("BRAIN_CREDENTIAL_EMAIL").unwrap_or_else(|_| {
        print!("Email: ");
        std::io::stdout().flush().unwrap();
        let mut input = String::new();
        std::io::stdin().read_line(&mut input).unwrap();
        input.trim().to_string()
    });

    let password = std::env::var("BRAIN_CREDENTIAL_PASSWORD").unwrap_or_else(|_| {
        rpassword::prompt_password("Password: ").expect("Failed to read password")
    });

    let creds = Credentials { email, password };

    // Save for next time
    std::fs::create_dir_all(secrets_path.parent().unwrap()).ok();
    let json = serde_json::to_string(&creds)?;
    std::fs::write(secrets_path, json).ok();

    Ok(creds)
}

pub async fn get_datasets(
    instrument_type: &str,
    region: &str,
    delay: i32,
    universe: &str,
    theme: Option<bool>,
) -> anyhow::Result<Vec<Dataset>> {
    let client = session::global();
    let url = format!("{}/data-sets", BRAIN_API_URL);

    let mut query_params = vec![
        ("instrumentType", instrument_type.to_string()),
        ("region", region.to_string()),
        ("delay", delay.to_string()),
        ("universe", universe.to_string()),
    ];

    if let Some(t) = theme {
        query_params.push(("theme", t.to_string()));
    }

    let res = client.get(url).query(&query_params).send().await?;
    let status = res.status();
    let json_body: serde_json::Value = res.json().await?;
    debug!("get_datasets response body: {}", json_body);
    let body: ResultsResponse<Dataset> = serde_json::from_value(json_body)?;
    info!("get_datasets status: {}, count: {}", status, body.count);
    Ok(body.results)
}

pub async fn get_datafields(
    instrument_type: &str,
    region: &str,
    delay: i32,
    universe: &str,
    search_opt: Option<String>,
) -> anyhow::Result<Vec<DataField>> {
    let client = session::global();
    let url = format!("{}/data-fields", BRAIN_API_URL);

    let mut query_params = vec![
        ("instrumentType", instrument_type.to_string()),
        ("region", region.to_string()),
        ("delay", delay.to_string()),
        ("universe", universe.to_string()),
    ];
    if let Some(search) = search_opt {
        info!(
            "Getting fields for: region={}, delay={}, universe={}, search={}",
            region, delay, universe, search
        );
        query_params.push(("search", search));
        query_params.push(("limit", "50".to_string()));
        query_params.push(("offset", "0".to_string()));
    } else {
        info!(
            "Getting fields for: region={}, delay={}, universe={}",
            region, delay, universe
        );
    }
    let res = client.get(url).query(&query_params).send().await?;
    let status = res.status();
    let json_body: serde_json::Value = res.json().await?;
    debug!("get_datafields response body: {}", json_body);
    let body: ResultsResponse<DataField> = serde_json::from_value(json_body)?;
    info!("get_datafields status: {}, count: {}", status, body.count);
    Ok(body.results)
}

pub async fn get_operators() -> anyhow::Result<Vec<Operator>> {
    let client = session::global();
    let url = format!("{}/operators", BRAIN_API_URL);
    let response = client.get(url).send().await?;
    let json_body: serde_json::Value = response.json().await?;
    debug!("get_operators response body: {}", json_body);
    let body: Vec<Operator> = serde_json::from_value(json_body)?;
    Ok(body)
}

pub fn generate_alpha(
    regular: Option<String>,
    selection: Option<String>,
    combo: Option<String>,
    alpha_type: &str,
    region: &str,
    universe: &str,
    delay: i32,
    decay: i32,
    neutralization: &str,
    truncation: f64,
    pasteurization: &str,
    test_period: &str,
    unit_handling: &str,
    nan_handling: &str,
    max_trade: &str,
    selection_handling: &str,
    selection_limit: i32,
    visualization: bool,
) -> Option<AlphaConfig> {
    let mut settings = Settings {
        instrument_type: "EQUITY".to_string(),
        region: region.to_string(),
        universe: universe.to_string(),
        delay,
        decay,
        neutralization: neutralization.to_string(),
        truncation,
        pasteurization: pasteurization.to_string(),
        test_period: test_period.to_string(),
        unit_handling: unit_handling.to_string(),
        nan_handling: nan_handling.to_string(),
        max_trade: max_trade.to_string(),
        language: "FASTEXPR".to_string(),
        visualization,
        selection_handling: None,
        selection_limit: None,
    };

    match alpha_type {
        "REGULAR" => Some(AlphaConfig::Regular {
            settings,
            regular: regular.unwrap_or_default(),
        }),
        "SUPER" => {
            settings.selection_handling = Some(selection_handling.to_string());
            settings.selection_limit = Some(selection_limit);
            Some(AlphaConfig::Super {
                settings,
                combo: combo.unwrap_or_default(),
                selection: selection.unwrap_or_default(),
            })
        }
        _ => {
            error!("alpha_type should be REGULAR or SUPER");
            None
        }
    }
}

type AsyncCallback = Box<dyn Fn(SimulationResult) -> Pin<Box<dyn Future<Output = ()> + Send>> + Send + Sync>;

pub struct BrainApi {
    callback: AsyncCallback
}

impl BrainApi {
    pub fn new<F, Fut>(callback: F) -> Self 
    where
        F: Fn(SimulationResult) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = ()> + Send + 'static
    {
        Self { 
            callback: Box::new(move |res| Box::pin(callback(res)))
        }
    }

    pub async fn run_callback(&self, result: SimulationResult) {
        (self.callback)(result).await;
    }

    pub async fn simulate_alpha_list_multi(
        &self,
        alpha_list: Vec<AlphaConfig>,
        mut limit_of_concurrent_simulations: usize,
        mut limit_of_multi_simulations: usize,
    ) -> Vec<SimulationResult>
    {
        if limit_of_multi_simulations < 2 || limit_of_multi_simulations > 10 {
            warn!("Limit of multi-simulation should be 2..10, will be set to 10");
            limit_of_multi_simulations = 10;
        }
        if limit_of_concurrent_simulations < 1 || limit_of_concurrent_simulations > 8 {
            warn!("Limit of concurrent simulation should be 1..8, will be set to 3");
            limit_of_concurrent_simulations = 3;
        }

        if alpha_list
            .iter()
            .any(|d| matches!(d, AlphaConfig::Super { .. }))
        {
            warn!(
                "Multi-Simulation is not supported for SuperAlphas, single concurrent simulations will be used"
            );
            // return simulate_alpha_list(client, alpha_list, 3).await;
        }

        let chunks: Vec<Vec<AlphaConfig>> = alpha_list
            .chunks(limit_of_multi_simulations)
            .map(|c| c.to_vec())
            .collect();

        // let pb = ProgressBar::new(chunks.len() as u64);
        // pb.set_style(
        //     ProgressStyle::default_bar()
        //         .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} ({eta})")
        //         .unwrap()
        //         .progress_chars("#>-"),
        // );

        let result_list = futures::stream::iter(chunks)
            .map(|chunk| {
                async move {
                    match self.simulate_multi_alpha(chunk).await {
                        Ok(res) => res,
                        Err(e) => {
                            error!("simulate_multi_alpha error: {}", e);
                            vec![]
                        }
                    }
                    // pb.inc(1);
                }
            })
            .buffer_unordered(limit_of_concurrent_simulations)
            .collect::<Vec<Vec<SimulationResult>>>()
            .await;

        // pb.finish_with_message("Done");

        result_list.into_iter().flatten().collect()
    }

    pub async fn simulate_multi_alpha(
        &self,
        simulate_data_list: Vec<AlphaConfig>,
    ) -> anyhow::Result<Vec<SimulationResult>> {
        if simulate_data_list.len() == 1 {
            let single_result = self
                .simulate_single_alpha(simulate_data_list[0].clone())
                .await?;
            return Ok(vec![single_result]);
        }

        let json_data = serde_json::to_value(&simulate_data_list)?;
        let simulate_response = self.start_simulation(&json_data).await?;
        let results = self.multisimulation_progress(simulate_response).await?;

        // if !completed {
        //     return simulate_data_list
        //         .into_iter()
        //         .map(|x| SimulationResult {
        //             alpha_id: None,
        //             simulate_data: x,
        //         })
        //         .collect();
        // }

        Ok(results)
    }

    pub async fn start_simulation(
        &self,
        data: &serde_json::Value,
    ) -> anyhow::Result<reqwest::Response> {
        let client = session::global();
        let url = format!("{}/simulations", BRAIN_API_URL);
        info!("request: {}", data);
        let response = client.post(url).json(data).send().await?;
        let status = response.status();
        // let body: serde_json::Value = response.json().await?;
        info!("start_simulation status: {}", status);
        Ok(response)
    }

    pub async fn multisimulation_progress(
        &self,
        response: reqwest::Response,
    ) -> anyhow::Result<Vec<SimulationResult>> {
        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            return anyhow::bail!("Simulation failed. {}, Status code: {}", text, status);
        }

        let progress_url = match response.headers().get(LOCATION) {
            Some(loc) => loc.to_str().unwrap_or_default().to_string(),
            None => return anyhow::bail!("Cannot find Location field in response header"),
        };

        let mut error_flag = false;
        let mut final_data = json!({});
        let client = session::global();

        loop {
            let prog_res = match client.get(progress_url.clone()).send().await {
                Ok(res) => res,
                Err(_) => {
                    tokio::time::sleep(Duration::from_secs(30)).await;
                    continue;
                }
            };

            if !prog_res.status().is_success() {
                tokio::time::sleep(Duration::from_secs(30)).await;
                continue;
            }

            let retry_after = prog_res
                .headers()
                .get("retry-after")
                .and_then(|h| h.to_str().ok())
                .and_then(|s| s.parse::<f64>().ok())
                .unwrap_or(0.0);

            let body: serde_json::Value = prog_res.json().await?;
            debug!("multisimulation progress body: {}", body);

            if retry_after == 0.0 {
                if body.get("status").and_then(|s| s.as_str()) == Some("ERROR") {
                    error_flag = true;
                }
                final_data = body;
                break;
            }
        }

        let children = final_data.get("children").and_then(|c| c.as_array());

        if error_flag {
            if let Some(children_arr) = children {
                for child in children_arr {
                    if let Some(child_id) = child.as_str() {
                        let response = self.get_simulation_progress_by_id(child_id).await?;
                        error!("Child Simulation failed: {:?}", response);
                    }
                }
            }
            return anyhow::bail!("Simulation failed. {:?}", final_data);
        }

        let children_arr = match children {
            Some(c) if !c.is_empty() => c,
            _ => {
                return anyhow::bail!("Multi-Simulation failed. {:?}", final_data);
            }
        };

        let mut results = Vec::new();
        for child in children_arr {
            if let Some(child_id) = child.as_str() {
                let response = self.get_simulation_progress_by_id(child_id).await?;
                let body: serde_json::Value = response.json().await?;
                info!("get_simulation_progress_by_id body: {}", body);
                if let Some(alpha) = body.get("alpha") {
                    let alpha_id = alpha.as_str().unwrap_or_default();
                    let regular = body.get("regular").unwrap().as_str().unwrap();
                    let result = SimulationResult {
                        alpha_id: alpha_id.to_string(),
                        regular: regular.to_string()
                    };
                    self.run_callback(result.clone()).await;
                    results.push(result);
                }
            }
        }

        Ok(results)
    }

    pub async fn get_simulation_progress_by_id(
        &self,
        sim_id: &str,
    ) -> anyhow::Result<reqwest::Response> {
        let client = session::global();
        let url = format!("{}/simulations/{}", BRAIN_API_URL, sim_id);
        let response = client.get(url).send().await?;
        Ok(response)
    }

    pub async fn simulate_single_alpha(
        &self,
        simulate_data: AlphaConfig,
    ) -> anyhow::Result<SimulationResult> {
        let json_data = serde_json::to_value(&simulate_data)?;
        let response = self.start_simulation(&json_data).await?;
        self.simulation_progress(response).await
    }

    pub async fn simulation_progress(&self, response: reqwest::Response) -> anyhow::Result<SimulationResult> {
        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            return anyhow::bail!("Simulation failed. {}, Status code: {}", text, status);
        }

        let progress_url = match response.headers().get(LOCATION) {
            Some(loc) => loc.to_str().unwrap_or_default().to_string(),
            None => return anyhow::bail!("Cannot find Location field in response header"),
        };

        let mut retry_count = 0;

        let client = session::global();
        loop {
            let prog_res = match client.get(progress_url.clone()).send().await {
                Ok(res) => res,
                Err(_) => {
                    tokio::time::sleep(Duration::from_secs(30)).await;
                    continue;
                }
            };

            if !prog_res.status().is_success() {
                error!(
                    "Simulation {}, Status code: {}, Retry",
                    progress_url,
                    prog_res.status()
                );
                tokio::time::sleep(Duration::from_secs(30)).await;
                retry_count += 1;
                if retry_count <= 2 {
                    continue;
                } else {
                    return anyhow::bail!(
                        "Simulation {} failed, Status code: {}",
                        progress_url,
                        prog_res.status()
                    );
                }
            }

            let retry_after = prog_res
                .headers()
                .get("retry-after")
                .and_then(|h| h.to_str().ok())
                .and_then(|s| s.parse::<f64>().ok())
                .unwrap_or(0.0);

            let body: serde_json::Value = prog_res.json().await?;
            debug!("simulation progress body: {}", body);

            if retry_after == 0.0 {
                if body.get("status").and_then(|s| s.as_str()) == Some("ERROR") {
                    return anyhow::bail!("Simulation failed. {:?}", body);
                }

                if let Some(alpha) = body.get("alpha") {
                    let alpha_id = alpha.as_str().unwrap_or_default();
                    let regular = body.get("regular").unwrap().as_str().unwrap_or_default();
                    return Ok(SimulationResult { alpha_id: alpha_id.to_string(), regular: regular.to_string()});
                } else {
                    return anyhow::bail!("Simulation failed. {:?}", body);
                }

                // let result = client.get_alpha(alpha_id).await.unwrap_or(json!({}));
                // if result.is_object() && !result.as_object().unwrap().is_empty() {
                //     return (true, result);
                // } else {
                //     return (false, json!({}));
                // }
            }

            // tokio::time::sleep(Duration::from_secs_f64(retry_after)).await;
        }
    }
}
