use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;
use std::time::Duration;

use futures_util::TryStreamExt;
use moka::future::Cache;
use rand::distributions::{Alphanumeric, DistString};
use reqwest::{multipart, Body, RequestBuilder, Response, StatusCode};
use serde::Serialize;
use sha1::{Digest, Sha1};
use thiserror::Error;
use tokio::fs::File;
use tokio_util::codec::{BytesCodec, FramedRead};

use crate::photo_service::PhotoPrismServiceError::{
    AddLabelFailed, AuthenticationError, CanNotFindPhotoByHash, IndexingFailed, UploadFailed,
    UserIDIsMissing, XSessionHeaderMissing,
};

#[derive(Debug, Clone)]
pub struct PhotoUID(pub String);

pub trait PhotoService {
    type Error;

    async fn upload_photo<P: AsRef<Path>>(&self, path: P) -> Result<PhotoUID, Self::Error>;

    async fn add_label(&self, photo_uid: &PhotoUID, label: &str) -> Result<(), Self::Error>;
}

#[derive(Error, Debug)]
pub enum PhotoPrismServiceError {
    #[error("Failed to authenticate at PhotoPrism service {0}.")]
    AuthenticationError(String),
    #[error("X-Session-ID header is missing.")]
    XSessionHeaderMissing,
    #[error("user.ID is missing in response json {0}.")]
    UserIDIsMissing(String),
    #[error("Failed to upload file {file} to PhotoPrism server: {details}.")]
    UploadFailed { file: String, details: String },
    #[error("Failed to index file {file} at PhotoPrism server: {details}.")]
    IndexingFailed { file: String, details: String },
    #[error(
        "Photo has been uploaded. But for some reason it is impossible to find it by hash {0}."
    )]
    CanNotFindPhotoByHash(String),
    #[error("Failed to add label {0} to file with uid {}", .photo_uid.0)]
    AddLabelFailed { label: String, photo_uid: PhotoUID },
    #[error("PhotoPrism API Error: {}", .err.to_string())]
    PhotoPrismAPIError {
        #[from]
        err: reqwest::Error,
    },
    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

#[derive(Serialize)]
#[serde(rename_all = "PascalCase")]
struct Label {
    name: String,
    priority: i32,
}

pub struct PhotoPrismPhotoService {
    photoprism_url: String,
    username: String,
    password: String,
    client: reqwest::Client,
    user_cache: Cache<(), Arc<PhotoPrismUser>>,
}

pub struct PhotoPrismUser {
    pub session_id: String,
    uid: String,
}

impl PhotoPrismPhotoService {
    pub fn new(
        photoprism_url: String,
        username: String,
        password: String,
        session_refresh_sec: u64,
    ) -> Self {
        let client = reqwest::Client::new();
        let user_cache = Cache::builder()
            .max_capacity(1)
            // Reload X-Session-ID every N seconds
            .time_to_live(Duration::from_secs(session_refresh_sec))
            .build();

        Self {
            photoprism_url,
            username,
            password,
            user_cache,
            client,
        }
    }

    pub async fn get_user(&self) -> Result<Arc<PhotoPrismUser>, PhotoPrismServiceError> {
        match self.user_cache.get(&()).await {
            None => {
                let user = Arc::new(self.authentication().await?);
                self.user_cache.insert((), user.clone()).await;
                Ok(user)
            }
            Some(v) => Ok(v),
        }
    }

    fn endpoint(&self, api_method: &str) -> String {
        format!("{}/api/v1{}", &self.photoprism_url, api_method)
    }

    async fn authentication(&self) -> Result<PhotoPrismUser, PhotoPrismServiceError> {
        // TODO: Use oauth2 when it will be ready https://github.com/photoprism/photoprism/issues/3943
        let params = HashMap::from([("username", &self.username), ("password", &self.password)]);

        let auth_resp = self
            .client
            .post(self.endpoint("/session"))
            .json(&params)
            .send()
            .await?;

        if auth_resp.status() != StatusCode::OK {
            return Err(AuthenticationError(auth_resp.text().await?));
        }

        let session_id = auth_resp
            .headers()
            .get("X-Session-ID")
            .map(|header| header.to_str().map_err(|e| anyhow::Error::from(e).into()))
            .unwrap_or_else(|| Err(XSessionHeaderMissing))?
            .to_owned();

        let user_data = auth_resp.json::<serde_json::Value>().await?;

        let uid = user_data
            .get("user")
            .and_then(|v| v.get("UID"))
            .and_then(|v| v.as_str().map(Ok))
            .unwrap_or_else(|| Err(UserIDIsMissing(user_data.to_string())))?
            .to_owned();

        Ok(PhotoPrismUser { session_id, uid })
    }

    async fn calculate_sha1<P: AsRef<Path>>(file_path: P) -> Result<String, anyhow::Error> {
        let file = File::open(file_path).await?;
        let mut hasher = Sha1::new();
        let mut stream = FramedRead::new(file, BytesCodec::new());
        while let Some(bytes) = stream.try_next().await? {
            hasher.update(&bytes);
        }

        Ok(hex::encode(hasher.finalize()))
    }

    pub async fn send(
        &self,
        request_builder: RequestBuilder,
    ) -> Result<Response, PhotoPrismServiceError> {
        // It is possible to use middleware for that, but for me, it does not make sense to use middleware for just one class.
        // It's better to have a separate method instead.
        let user = self.get_user().await?;
        let response = request_builder
            .header("X-Session-ID", &user.session_id)
            .send()
            .await?;
        Ok(response)
    }

    async fn search_photo_by_hash(
        &self,
        file_hash: &String,
    ) -> Result<Option<PhotoUID>, PhotoPrismServiceError> {
        let search_by_hash = format!("quality:-100 hash:{}", file_hash);
        let search_params: Vec<(&str, &str)> = vec![
            ("q", search_by_hash.as_str()),
            ("count", "1"),
            ("order", "newest"),
        ];

        let search_file_response = self
            .send(
                self.client
                    .get(self.endpoint("/photos"))
                    .query(&search_params),
            )
            .await?;

        let photos = search_file_response.json::<serde_json::Value>().await?;

        Ok(photos
            .as_array()
            .and_then(|v| if v.is_empty() { None } else { v[0].get("UID") })
            .and_then(|v| v.as_str())
            .map(|v| PhotoUID(v.to_owned())))
    }
}

impl PhotoService for PhotoPrismPhotoService {
    type Error = PhotoPrismServiceError;
    async fn upload_photo<P: AsRef<Path>>(&self, file_path: P) -> Result<PhotoUID, Self::Error> {
        let random_token = Alphanumeric.sample_string(&mut rand::thread_rng(), 6);
        let extension: &str = file_path
            .as_ref()
            .extension()
            .and_then(|v| v.to_str())
            .unwrap();
        let file = File::open(&file_path).await.map_err(anyhow::Error::from)?;
        let file_body: Body = Body::wrap_stream(FramedRead::new(file, BytesCodec::new()));
        //create the multipart form
        let form = multipart::Form::new().part(
            "files",
            multipart::Part::stream(file_body).file_name(format!("unknown.{}", extension)),
        );
        let user_uid = &self.get_user().await?.uid;
        let upload_http_endpoint =
            self.endpoint(&format!("/users/{}/upload/{}", user_uid, random_token));
        let upload_file_resp = self
            .send(self.client.post(&upload_http_endpoint).multipart(form))
            .await?;

        if upload_file_resp.status() != StatusCode::OK {
            return Err(UploadFailed {
                file: file_path.as_ref().to_str().unwrap().to_owned(),
                details: upload_file_resp.text().await?,
            });
        }

        let album_json: serde_json::Value = serde_json::from_str(r#"{"albums": []}"#).unwrap();
        let process_upload_file_resp = self
            .send(self.client.put(upload_http_endpoint).json(&album_json))
            .await?;

        if process_upload_file_resp.status() != StatusCode::OK {
            return Err(IndexingFailed {
                file: file_path.as_ref().to_str().unwrap().to_owned(),
                details: process_upload_file_resp.text().await?,
            });
        }

        let file_hash = Self::calculate_sha1(file_path).await?;

        match self.search_photo_by_hash(&file_hash).await? {
            Some(photo_uid) => Ok(photo_uid),
            None => Err(CanNotFindPhotoByHash(file_hash)),
        }
    }

    async fn add_label(&self, photo_uid: &PhotoUID, label: &str) -> Result<(), Self::Error> {
        let add_label_http_endpoint = self.endpoint(&format!("/photos/{}/label", photo_uid.0));
        let add_label_params = Label {
            name: label.to_owned(),
            priority: 10,
        };
        let add_label_response = self
            .send(
                self.client
                    .post(&add_label_http_endpoint)
                    .json(&add_label_params),
            )
            .await?;

        if add_label_response.status() != StatusCode::OK {
            Err(AddLabelFailed {
                label: label.to_owned(),
                photo_uid: (*photo_uid).to_owned(),
            })
        } else {
            Ok(())
        }
    }
}
