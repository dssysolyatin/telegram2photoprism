use std::marker::PhantomPinned;
use std::pin::Pin;
use std::time::Duration;

use anyhow::anyhow;
use futures_util::future::BoxFuture;
use futures_util::FutureExt;
use log::debug;
use testcontainers::{clients, Container, RunnableImage};
use tokio::time::sleep;

use telegram2photoprism::PhotoPrismPhotoService;
use telegram2photoprism::PhotoService;

use crate::photoprism_container::PhotoPrismContainer;

mod photoprism_container;

struct Fixture<'a> {
    docker_client: clients::Cli,
    photoprism_container: Option<Container<'a, PhotoPrismContainer>>,
    _marker: PhantomPinned,
}

/*  The current implementation of Fixture is overkill for tests.
The sole reason for this is to practice with a new language. */
impl Fixture<'_> {
    fn new() -> Pin<Box<Self>> {
        let this = Self {
            docker_client: clients::Cli::default(),
            photoprism_container: None,
            _marker: PhantomPinned,
        };

        let mut boxed = Box::pin(this);
        let ptr: *const clients::Cli = &boxed.docker_client;

        unsafe {
            let photoprism_container =
                (*ptr).run(RunnableImage::from(PhotoPrismContainer::default()));
            boxed.as_mut().get_unchecked_mut().photoprism_container = Some(photoprism_container);
        };

        boxed
    }
    fn get_photoprism_url(&self) -> String {
        let port = self
            .photoprism_container
            .as_ref()
            .unwrap()
            .get_host_port_ipv4(2342);
        format!("http://127.0.0.1:{}", port)
    }
}

async fn with_fixture<F>(f: F) -> Result<(), anyhow::Error>
where
    F: for<'a> FnOnce(&'a Fixture<'a>) -> BoxFuture<'a, Result<(), anyhow::Error>>,
{
    let _ = pretty_env_logger::try_init();
    let fixture = Fixture::new();
    f(&fixture).await
}

#[tokio::test(flavor = "multi_thread")]
async fn test_add_photo_add_labels() -> Result<(), anyhow::Error> {
    with_fixture(|fixture: &Fixture| {
        async {
            let photoprism_url = fixture.get_photoprism_url().to_owned();

            let photoprism_service = PhotoPrismPhotoService::new(
                photoprism_url.clone(),
                "admin".to_owned(),
                "insecure".to_owned(),
                3600,
            );

            let file_path = concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/resources/tests/low_quality_photo.jpg"
            );
            let photo_uid = photoprism_service.upload_photo(file_path).await?;
            photoprism_service.add_label(&photo_uid, "Han").await?;
            photoprism_service.add_label(&photo_uid, "Luke").await?;
            photoprism_service.add_label(&photo_uid, "Vader").await?;

            let client = reqwest::Client::new();
            let get_photo_endpoint = format!("{}/api/v1/photos/{}", photoprism_url, photo_uid.0);
            // check labels:
            let get_photo_json = photoprism_service
                .send(client.get(get_photo_endpoint))
                .await?
                .json::<serde_json::Value>()
                .await?;

            let labels = get_photo_json["Labels"].as_array().unwrap();
            assert_ne!(labels.iter().find(|x| x["Label"]["Name"] == "Han"), None);
            assert_ne!(labels.iter().find(|x| x["Label"]["Name"] == "Luke"), None);
            assert_ne!(labels.iter().find(|x| x["Label"]["Name"] == "Vader"), None);
            Ok(())
        }
        .boxed()
    })
    .await
}

#[tokio::test(flavor = "multi_thread")]
async fn test_refresh_token() -> Result<(), anyhow::Error> {
    with_fixture(|fixture: &Fixture| {
        async {
            let photoprism_url = fixture.get_photoprism_url().to_owned();
            let photoprism_service = PhotoPrismPhotoService::new(
                photoprism_url.clone(),
                "admin".to_owned(),
                "insecure".to_owned(),
                1,
            );

            let user_before_reload = photoprism_service.get_user().await?;
            let mut retries = 10;
            loop {
                retries -= 1;
                sleep(Duration::from_secs(1)).await;
                let user_after_reload = photoprism_service.get_user().await?;

                if user_before_reload.access_token != user_after_reload.access_token {
                    debug!(
                        "old session id: {}, new session id: {}",
                        user_before_reload.access_token, user_after_reload.access_token
                    );
                    break;
                }

                if retries == 0 {
                    Err(anyhow!(
                        "Session id has not been refreshing during 10 seconds"
                    ))?;
                }
            }

            Ok(())
        }
        .boxed()
    })
    .await
}
