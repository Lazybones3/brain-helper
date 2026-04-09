use brain_api::{self, AlphaConfig, Settings};
use brain_database::{alpha_dao, entity::AlphaEntity};
use jiff::Zoned;

fn alpha_config_to_entity(alpha_config: &AlphaConfig, alpha_id: Option<String>) -> AlphaEntity {
    match alpha_config {
        AlphaConfig::Regular { settings, regular } => AlphaEntity {
            regular: regular.clone(),
            instrument_type: settings.instrument_type.clone(),
            region: settings.region.clone(),
            universe: settings.universe.clone(),
            delay: settings.delay,
            decay: settings.decay,
            neutralization: settings.neutralization.clone(),
            truncation: settings.truncation.try_into().unwrap_or_default(),
            pasteurization: settings.pasteurization.clone(),
            test_period: settings.test_period.clone(),
            unit_handling: settings.unit_handling.clone(),
            nan_handling: settings.nan_handling.clone(),
            max_trade: settings.max_trade.clone(),
            language: settings.language.clone(),
            visualization: settings.visualization,
            selection_handling: settings.selection_handling.clone().unwrap_or_default(),
            selection_limit: settings.selection_limit.unwrap_or_default(),
            combo: "".to_string(),
            selection: "".to_string(),
            id: alpha_id.unwrap_or_default(),
            crete_time: Zoned::now().datetime(),
        },
        AlphaConfig::Super {
            settings,
            combo,
            selection,
        } => AlphaEntity {
            regular: "".to_string(),
            instrument_type: settings.instrument_type.clone(),
            region: settings.region.clone(),
            universe: settings.universe.clone(),
            delay: settings.delay,
            decay: settings.decay,
            neutralization: settings.neutralization.clone(),
            truncation: settings.truncation.try_into().unwrap_or_default(),
            pasteurization: settings.pasteurization.clone(),
            test_period: settings.test_period.clone(),
            unit_handling: settings.unit_handling.clone(),
            nan_handling: settings.nan_handling.clone(),
            max_trade: settings.max_trade.clone(),
            language: settings.language.clone(),
            visualization: settings.visualization,
            selection_handling: settings.selection_handling.clone().unwrap_or_default(),
            selection_limit: settings.selection_limit.unwrap_or_default(),
            combo: combo.clone(),
            selection: selection.clone(),
            id: alpha_id.unwrap_or_default(),
            crete_time: Zoned::now().datetime(),
        },
    }
}

pub async fn simulation() -> anyhow::Result<()> {
    let settings = Settings::default();
    brain_api::start_session().await?;
    let result = brain_api::get_datafields(
        &settings.instrument_type,
        &settings.region,
        settings.delay,
        &settings.universe,
        "Earnings forecasts",
    )
    .await?;
    let mut alpha_list = Vec::new();
    for item in result {
        if item.field_type == "MATRIX" {
            let regular = item.id;
            println!("{}", regular);
            let alpha_config = AlphaConfig::Regular {
                settings: settings.clone(),
                regular,
            };
            let entity = alpha_config_to_entity(&alpha_config, None);
            let flag = alpha_dao::check_if_exist(&entity).await?;
            if !flag {
                alpha_list.push(alpha_config);
            }
        }
    }
    if alpha_list.is_empty() {
        println!("alpha list is empty");
        return Ok(());
    }
    let api = brain_api::BrainApi::new(move |result| {
        let value = settings.clone();
        async move {
            let alpha_config = AlphaConfig::Regular {
                settings: value,
                regular: result.regular.clone(),
            };
            let entity = alpha_config_to_entity(&alpha_config, Some(result.alpha_id));
            let _ = alpha_dao::create(entity).await.unwrap();
        }
    });
    let _ = api.simulate_alpha_list_multi(alpha_list, 3, 3).await;

    Ok(())
}
