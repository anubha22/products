use super::{
    models::{StorageClientError, StorageFile},
    GetBlob, StorageClient,
};
use async_trait::async_trait;
use azure_sdk_core::{
    BlobNameSupport, BodySupport, ContainerNameSupport, ContentMD5Support, ContentTypeSupport,
    MetadataSupport,
};
use azure_sdk_storage_blob::Blob;
use azure_sdk_storage_core::client::Client;
use std::collections::HashMap;

pub struct AzureBlobStorage {
    pub container_name: String,
    prefix: String,
    storage_account: String,
    master_key: String,
}

impl AzureBlobStorage {
    pub fn temporary() -> Self {
        let container_name = std::env::var("STORAGE_CONTAINER_TEMPORARY")
            .expect("Set env variable STORAGE_CONTAINER_TEMPORARY first!");
        let storage_account =
            std::env::var("STORAGE_ACCOUNT").expect("Set env variable STORAGE_ACCOUNT first!");
        let master_key = std::env::var("STORAGE_MASTER_KEY")
            .expect("Set env variable STORAGE_MASTER_KEY first!");

        Self {
            container_name,
            prefix: "temp/".to_owned(),
            storage_account,
            master_key,
        }
    }
    pub fn permanent() -> Self {
        let container_name =
            std::env::var("STORAGE_CONTAINER").expect("Set env variable STORAGE_CONTAINER first!");
        let storage_account =
            std::env::var("STORAGE_ACCOUNT").expect("Set env variable STORAGE_ACCOUNT first!");
        let master_key = std::env::var("STORAGE_MASTER_KEY")
            .expect("Set env variable STORAGE_MASTER_KEY first!");

        Self {
            container_name,
            prefix: "".to_owned(),
            storage_account,
            master_key,
        }
    }

    pub fn log() -> Self {
        let container_name = std::env::var("LOG_STORAGE_CONTAINER")
            .expect("Set env variable LOG_STORAGE_CONTAINER first!");
        let storage_account = std::env::var("LOG_STORAGE_ACCOUNT")
            .expect("Set env variable LOG_STORAGE_ACCOUNT first!");
        let master_key = std::env::var("LOG_STORAGE_MASTER_KEY")
            .expect("Set env variable LOG_STORAGE_MASTER_KEY first!");

        Self {
            container_name,
            prefix: "".to_owned(),
            storage_account,
            master_key,
        }
    }

    pub fn get_azure_client(&self) -> Result<Client, StorageClientError> {
        let client = base64::decode(&self.master_key)
            .map(|_| Client::new(&self.storage_account, &self.master_key))?;

        Ok(client?)
    }
}

#[async_trait]
impl StorageClient for AzureBlobStorage {
    async fn add_file(
        &self,
        file_data: &[u8],
        licence_number: &str,
        metadata_ref: HashMap<&str, &str>,
    ) -> Result<StorageFile, StorageClientError> {
        let storage_client = self.get_azure_client()?;

        let file_digest = md5::compute(&file_data[..]);
        let name = format!("{}{}", &self.prefix, file_name(licence_number, file_data));

        storage_client
            .put_block_blob()
            .with_container_name(&self.container_name)
            .with_blob_name(&name)
            .with_content_type("application/pdf")
            .with_metadata(&metadata_ref)
            .with_body(&file_data[..])
            .with_content_md5(&file_digest[..])
            .finalize()
            .await
            .map_err(|e| {
                tracing::error!("Error uploading file to blob storage: {:?}", e);
                StorageClientError::UploadError(format!("Couldn't create blob: {:?}", e))
            })?;

        let path = format!(
            "https://{}.blob.core.windows.net/{}/{}",
            &self.storage_account, &self.container_name, &name
        );

        Ok(StorageFile { name, path })
    }
    async fn get_file(&self, storage_file: StorageFile) -> Result<Vec<u8>, StorageClientError> {
        let file_data = self
            .get_blob(&storage_file.name)
            .await
            .map_err(|e| {
                tracing::error!("Error retrieving file from blob storage: {:?}", e);
                StorageClientError::RetrievalError(format!("Couldn't retrieve blob: {:?}", e))
            })?
            .data;

        Ok(file_data)
    }
    async fn append_to_file(&self, file_name: &str, body: &[u8]) -> Result<(), StorageClientError> {
        let storage_client = self.get_azure_client()?;
        storage_client
            .put_append_block()
            .with_container_name(&self.container_name)
            .with_blob_name(&file_name)
            .with_body(body)
            .finalize()
            .await
            .map_err(|e| {
                tracing::error!("Error appending data to blob file: {:?}", e);
                StorageClientError::AppendError(format!(
                    "Couldn't append data to blob file: {:?}",
                    e
                ))
            })?;
        Ok(())
    }
}

fn file_name(licence_number: &str, file_data: &[u8]) -> String {
    let mut hash = sha1::Sha1::new();
    hash.update(licence_number.as_bytes());
    hash.update(file_data);
    hash.digest().to_string()
}
