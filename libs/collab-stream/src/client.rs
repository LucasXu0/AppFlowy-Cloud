use crate::error::StreamError;
use crate::pubsub::{CollabStreamPub, CollabStreamSub};
use crate::stream::CollabStream;
use crate::stream_group::{StreamConfig, StreamGroup};
use redis::aio::ConnectionManager;
use tracing::error;

pub const CONTROL_STREAM_KEY: &str = "af_collab_control";

#[derive(Clone)]
pub struct CollabRedisStream {
  connection_manager: ConnectionManager,
}

impl CollabRedisStream {
  pub async fn new(redis_client: redis::Client) -> Result<Self, redis::RedisError> {
    let connection_manager = redis_client.get_connection_manager().await?;
    Ok(Self::new_with_connection_manager(connection_manager))
  }

  pub fn new_with_connection_manager(connection_manager: ConnectionManager) -> Self {
    Self { connection_manager }
  }

  pub async fn stream(&self, workspace_id: &str, oid: &str) -> CollabStream {
    CollabStream::new(workspace_id, oid, self.connection_manager.clone())
  }

  pub async fn collab_control_stream(
    &self,
    key: &str,
    group_name: &str,
  ) -> Result<StreamGroup, StreamError> {
    let mut group = StreamGroup::new_with_config(
      key.to_string(),
      group_name,
      self.connection_manager.clone(),
      StreamConfig::new().with_max_len(1000),
    );

    // don't return error when create consumer group failed
    if let Err(err) = group.ensure_consumer_group().await {
      error!("Failed to ensure consumer group: {}", err);
    }

    Ok(group)
  }

  pub async fn collab_update_stream(
    &self,
    workspace_id: &str,
    oid: &str,
    group_name: &str,
  ) -> Result<StreamGroup, StreamError> {
    let stream_key = format!("af_collab_update-{}-{}", workspace_id, oid);
    let mut group = StreamGroup::new_with_config(
      stream_key,
      group_name,
      self.connection_manager.clone(),
      StreamConfig::new()
        // 2000 messages
        .with_max_len(2000)
        // 12 hours
        .with_expire_time(60 * 60 * 12),
    );
    group.ensure_consumer_group().await?;
    Ok(group)
  }
}

pub struct PubSubClient {
  redis_client: redis::Client,
  connection_manager: ConnectionManager,
}

impl PubSubClient {
  pub async fn new(redis_client: redis::Client) -> Result<Self, redis::RedisError> {
    let connection_manager = redis_client.get_connection_manager().await?;
    Ok(Self {
      redis_client,
      connection_manager,
    })
  }

  pub async fn collab_pub(&self) -> CollabStreamPub {
    CollabStreamPub::new(self.connection_manager.clone())
  }

  #[allow(deprecated)]
  pub async fn collab_sub(&self) -> Result<CollabStreamSub, StreamError> {
    let conn = self.redis_client.get_async_connection().await?;
    Ok(CollabStreamSub::new(conn))
  }
}
