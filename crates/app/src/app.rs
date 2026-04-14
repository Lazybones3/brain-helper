use brain_api::{self, AlphaConfig, Settings, response::datafield::DataField};
use brain_database::{alpha_dao, entity::AlphaEntity};
use futures::StreamExt;
use jiff::Zoned;
use tracing::{error, info, warn};

pub struct BrainAppCore {
    settings: Settings,
    theme: Option<bool>,
}

impl BrainAppCore {
    pub async fn new(settings: Settings) -> anyhow::Result<Self> {
        brain_api::start_session().await?;
        Ok(Self {
            settings,
            theme: None,
        })
    }

    pub async fn get_fields_by_dataset(&self, dataset_name: &str) -> anyhow::Result<Vec<DataField>> {
        let datafields = brain_api::get_datafields(
            &self.settings.instrument_type,
            &self.settings.region,
            self.settings.delay,
            &self.settings.universe,
            None,
        )
        .await?;
        let result = datafields.into_iter()
            .filter(|item| item.dataset.name.eq(&dataset_name))
            .collect::<Vec<DataField>>();
        Ok(result)
    }

    pub async fn simulation(&self, dataset_name: &str, alpha_list: Vec<AlphaConfig>) -> anyhow::Result<()> {
        if alpha_list.is_empty() {
            warn!("alpha list is empty");
            return Ok(());
        }
        info!("Total alpha: {}", alpha_list.len());
        let filtered_alpha_list = futures::stream::iter(alpha_list).filter_map(|item| async move {
            let entity = alpha_config_to_entity(&item, None, None);
            match alpha_dao::check_if_exist(&entity).await {
                Ok(exists) =>  {
                    if !exists {
                        return Some(item);
                    }
                },
                Err(e) => { error!("Filter alpha list error: {}", e); }
            }
            None
        }).collect::<Vec<AlphaConfig>>().await;
        info!("Filtered total alpha: {}", filtered_alpha_list.len());

        let settings_clone = self.settings.clone();
        let dataset_clone = dataset_name.to_string();
        let api = brain_api::BrainApi::new(move |result| {
            let value = settings_clone.clone();
            let dataset_name_value = dataset_clone.clone();
            async move {
                let alpha_config = AlphaConfig::Regular {
                    settings: value,
                    regular: result.regular.clone(),
                };
                let entity = alpha_config_to_entity(&alpha_config, Some(result.alpha_id), Some(dataset_name_value));
                let _ = alpha_dao::create(entity).await.unwrap();
            }
        });
        let _ = api.simulate_alpha_list_multi(filtered_alpha_list, 3, 3).await;

        Ok(())
    }
}

fn alpha_config_to_entity(alpha_config: &AlphaConfig, alpha_id: Option<String>, dataset: Option<String>) -> AlphaEntity {
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
            dataset: dataset.unwrap_or_default(),
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
            dataset: dataset.unwrap_or_default(),
            crete_time: Zoned::now().datetime(),
        },
    }
}
