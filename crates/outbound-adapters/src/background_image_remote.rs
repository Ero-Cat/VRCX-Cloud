use std::sync::Arc;

use chrono::Utc;
use vrcx_0_application::profile::{
    BackgroundImageProviderId, BackgroundImageRemote, BackgroundImageRemoteFuture,
};
use vrcx_0_application_core::{Error, WebClient};
use vrcx_0_contracts::background_image as protocol;
use vrcx_0_integrations::external_api::{self, ExternalApiScope};

pub struct ExternalBackgroundImageRemote {
    web: Arc<WebClient>,
}

impl ExternalBackgroundImageRemote {
    pub fn new(web: Arc<WebClient>) -> Self {
        Self { web }
    }

    async fn fetch_json(&self, url: &str) -> crate::Result<(i32, String)> {
        let response = self
            .web
            .execute_external_api(
                external_api::background_image_get_input(url),
                ExternalApiScope::BackgroundImage,
            )
            .await?;
        Ok((response.status, response.data))
    }
}

impl BackgroundImageRemote for ExternalBackgroundImageRemote {
    fn provider_image(
        &self,
        provider_id: BackgroundImageProviderId,
    ) -> BackgroundImageRemoteFuture<'_, protocol::BackgroundImageProviderImage> {
        Box::pin(async move {
            let date_key = Utc::now().format("%Y-%m-%d").to_string();
            match provider_id {
                BackgroundImageProviderId::NasaEpic => {
                    let (status, body) = self.fetch_json(protocol::NASA_EPIC_METADATA_URL).await?;
                    ensure_provider_status(status)?;
                    protocol::parse_nasa_epic_response(&body)
                        .map_err(|error| Error::Custom(error.to_string()))
                }
                BackgroundImageProviderId::AicPublicDomain => {
                    let (status, body) = self
                        .fetch_json(protocol::AIC_PUBLIC_DOMAIN_SEARCH_URL)
                        .await?;
                    ensure_provider_status(status)?;
                    protocol::parse_aic_response(&body, &date_key)
                        .map_err(|error| Error::Custom(error.to_string()))
                }
                BackgroundImageProviderId::NasaApodSafe => {
                    let today = Utc::now();
                    for offset in 0..=protocol::NASA_APOD_IMAGE_LOOKBACK_DAYS {
                        let date = (today - chrono::Duration::days(offset as i64))
                            .format("%Y-%m-%d")
                            .to_string();
                        let (status, body) = self
                            .fetch_json(&protocol::nasa_apod_request_url(&date))
                            .await?;
                        if status == 404 {
                            continue;
                        }
                        ensure_provider_status(status)?;
                        if let Some(image) = protocol::parse_nasa_apod_response(&body, &date_key)
                            .map_err(|error| Error::Custom(error.to_string()))?
                        {
                            return Ok(image);
                        }
                    }
                    Err(Error::Custom(
                        "NASA APOD did not return a copyright-free image in the recent archive."
                            .into(),
                    ))
                }
            }
        })
    }
}

fn ensure_provider_status(status: i32) -> crate::Result<()> {
    if status == 429 {
        return Err(Error::Custom(
            "Background Image provider rate limit reached.".into(),
        ));
    }
    if !(200..300).contains(&status) {
        return Err(Error::Custom(format!(
            "Failed to load Background Image provider: {status}"
        )));
    }
    Ok(())
}
