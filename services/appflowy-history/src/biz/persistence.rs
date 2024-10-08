use crate::biz::snapshot::{CollabSnapshot, CollabSnapshotState};
use crate::error::HistoryError;
use collab_entity::CollabType;
use database::history::ops::insert_history;
use sqlx::PgPool;
use tonic_proto::history::SnapshotMetaPb;
use tracing::info;
use uuid::Uuid;

pub struct HistoryPersistence {
  workspace_id: Uuid,
  pg_pool: PgPool,
}

impl HistoryPersistence {
  pub fn new(workspace_id: Uuid, pg_pool: PgPool) -> Self {
    Self {
      workspace_id,
      pg_pool,
    }
  }
  pub async fn insert_history(
    &self,
    state: CollabSnapshotState,
    snapshots: Vec<CollabSnapshot>,
    collab_type: CollabType,
  ) -> Result<(), HistoryError> {
    info!(
      "[History]: write {}:{}: {} snapshots, doc state len:{}",
      state.object_id,
      collab_type,
      snapshots.len(),
      state.doc_state.len(),
    );

    let snapshots = snapshots
      .into_iter()
      .map(SnapshotMetaPb::from)
      .collect::<Vec<_>>();

    insert_history(
      &self.workspace_id,
      &state.object_id,
      state.doc_state,
      state.doc_state_version,
      None,
      collab_type,
      state.created_at,
      snapshots,
      self.pg_pool.clone(),
    )
    .await?;
    Ok(())
  }
}
