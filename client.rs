use {
    crate::plugin_manager,
    leptos::{view, IntoView, View},
    types::timing::TimeRange,
    url::Url,
    crate::api::encode_url_component
};
pub struct Plugin {}

impl plugin_manager::Plugin for Plugin {
    fn get_style(&self) -> plugin_manager::Style {
        plugin_manager::Style::Acc2
    }
    async fn new(_data: plugin_manager::PluginData) -> Self
    where
        Self: Sized,
    {
        Plugin {}
    }
    fn get_component(
        &self,
        data: plugin_manager::PluginEventData,
    ) -> crate::plugin_manager::EventResult<Box<dyn FnOnce() -> leptos::View>> {
        let (range, mut url, signature) = data.get_data::<(TimeRange, Url, String)>()?;
        url.set_path("/observe/");
        url.set_query(Some(&format!(
            "skipWelcome=true&start={}&end={}&signature={}",
            range.start.timestamp_millis(),
            range.end.timestamp_millis(),
            encode_url_component(&signature)
        )));

        Ok(Box::new(move || -> View {
            view! {
                <iframe
                    style:height="250px"
                    style:width="100%"
                    style:border="none"
                    class="wrapper"
                    src=move || url.to_string()
                >
                    Loading iframe
                </iframe>
            }.into_view()
        }))
    }
}
