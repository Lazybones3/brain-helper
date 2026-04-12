use brain_api::{self, AlphaConfig, Settings, response::datafield::DataField};
use brain_database::{alpha_dao, entity::AlphaEntity};
use jiff::Zoned;

pub struct BrainApp {
    settings: Settings,
    theme: Option<bool>,
}

impl BrainApp {
    pub async fn new(settings: Settings) -> anyhow::Result<Self> {
        brain_api::start_session().await?;
        Ok(Self {
            settings,
            theme: None,
        })
    }

    pub async fn get_fields_by_dataset(&self, dataset_name: &str, field_type_opt: Option<String>) -> anyhow::Result<Vec<DataField>> {
        let field_type = field_type_opt.unwrap_or("MATRIX".to_string());
        let datafields = brain_api::get_datafields(
            &self.settings.instrument_type,
            &self.settings.region,
            self.settings.delay,
            &self.settings.universe,
            &dataset_name,
        )
        .await?;
        let result = datafields.into_iter()
            .filter(|item| item.dataset.name.eq(&dataset_name) && item.field_type.eq(&field_type))
            .collect::<Vec<DataField>>();
        Ok(result)
    }

    pub async fn simulation(&self, dataset_name: &str, field_type_opt: Option<String>, template: Option<String>) -> anyhow::Result<()> {
        let result = self.get_fields_by_dataset(dataset_name, field_type_opt).await?;
        let mut alpha_list = Vec::new();
        for item in result {
            let regular = match template {
                Some(ref t) => t.replace("<field>", &item.id),
                None => item.id
            };
            println!("regular: {}", regular);
            let alpha_config = AlphaConfig::Regular {
                settings: self.settings.clone(),
                regular,
            };
            let entity = alpha_config_to_entity(&alpha_config, None, None);
            let flag = alpha_dao::check_if_exist(&entity).await?;
            if !flag {
                alpha_list.push(alpha_config);
            }
        }
        if alpha_list.is_empty() {
            println!("alpha list is empty");
            return Ok(());
        }
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
        let _ = api.simulate_alpha_list_multi(alpha_list, 3, 3).await;

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
