use {
    crate::PluginData,
    chrono::{DateTime, TimeDelta},
    serde::Deserialize,
    serde_json::json,
    types::{api::CompressedEvent, timing::TimeRange},
    url::Url
};

#[derive(Deserialize)]
struct ConfigData{
    pub password: String,
    pub url: Url
}

pub struct Plugin {
    _plugin_data: PluginData,
    config: ConfigData
}

impl crate::Plugin for Plugin {
    async fn new(data: PluginData) -> Self
    where
        Self: Sized,
    {
        let config: ConfigData = toml::Value::try_into(
            data.config
                .clone().expect("Failed to init location plugin! No config was provided!")
        )
        .unwrap_or_else(|e| panic!("Unable to init location plugin! Provided config does not fit the requirements: {}", e));

        Plugin { _plugin_data: data, config }
    }

    fn get_type() -> types::api::AvailablePlugins
    where
        Self: Sized,
    {
        types::api::AvailablePlugins::timeline_plugin_location
    }

    fn get_compressed_events (&self, query_range: &types::timing::TimeRange) -> std::pin::Pin<Box<dyn futures::Future<Output = types::api::APIResult<Vec<types::api::CompressedEvent>>> + Send>> {
        let url = self.config.url.clone();
        let password = self.config.password.clone();
        let query_range = query_range.clone();
        Box::pin(async move {
            let mut api_url = url.clone();
            api_url.set_path("/api/data");
            let client = reqwest::Client::new();
            let request_result = client.post(api_url).body(json! ({"export": "timespan", "module": "public/locations.js", "arguments": [password]}).to_string()).header("Content-Type", "application/json").send().await;
            let timespan = match request_result {
                Ok(v) => {
                    match v.text().await {
                        Ok(v) => {
                            let timespan_response = serde_json::from_str::<TimespanResponse>(&v)?;
                            TimeRange {
                                start: DateTime::from_timestamp_millis(timespan_response.data.start as i64).unwrap(),
                                end: DateTime::from_timestamp_millis(timespan_response.data.end as i64).unwrap(),
                            }
                        }
                        Err(e) => {
                            return Err(types::api::APIError::PluginError(format!("Location Plugin: Unable to get timespan: Request Error: {}", e)))
                        }
                    }
                }
                Err(e) => {
                    return Err(types::api::APIError::PluginError(format!("Location Plugin: Unable to get timespan: Request Error: {}", e)))
                }
            };

            let mut resulting_vec = Vec::new();
            let mut current = query_range.start;

            while current < query_range.end {
                let new_current = current.checked_add_signed(TimeDelta::try_hours(1).unwrap()).unwrap();
                if timespan.includes(&current) {
                    resulting_vec.push(TimeRange {
                        start: current,
                        end: new_current
                    });
                }
                current = new_current;
            }
            let resulting_vec: Vec<_> = resulting_vec.into_iter().map(|v| {
                CompressedEvent {
                    time: types::timing::Timing::Range(v.clone()),
                    data: Box::new((v, url.clone())),
                    title: "Locations".to_string()
                }
            }).collect();
            
            Ok(resulting_vec)
        })
    }
}

#[derive(Deserialize)]
struct TimespanResponse {
    pub success: bool,
    pub data: TimespanResponseData
}

#[derive(Deserialize)]
struct TimespanResponseData {
    pub success: bool,
    pub start: u64,
    pub end: u64
}