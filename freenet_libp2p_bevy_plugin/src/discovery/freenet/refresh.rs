use crate::discovery;

pub async fn refresh(
    directory_client: &mut discovery::freenet::DirectoryClient,
) -> Result<discovery::Directory, discovery::Error> {
    let instance_id = *directory_client.key.id();
    let fresh = discovery::freenet::fetch(&mut directory_client.client, instance_id).await?;
    let cached = std::mem::take(&mut directory_client.directory);
    directory_client.directory = discovery::freenet::merge_directory(cached, fresh);
    Ok(directory_client.directory.clone())
}

// no test_usage necessary
