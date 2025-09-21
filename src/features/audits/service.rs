use super::{
    types::AuditEventDto,
    db::{AuditEvent, EventType, LogLevel},
    repo::AuditRepository
};
use uuid::Uuid;

#[derive(Clone)]
pub struct AuditService {
    repo: AuditRepository,
}

impl AuditService {
    pub fn new(repo: AuditRepository) -> Self {
        Self { repo }
    }

    pub async fn create_event(
        &self,
        user_id: i64,
        event_type: EventType,
        log_level: LogLevel,
        session_id: Option<Uuid>,
    ) -> sqlx::Result<AuditEventDto> {
        let entity: AuditEvent = self
            .repo
            .create(user_id, event_type, log_level, session_id)
            .await?;
        Ok(entity.into())
    }

    pub async fn get_event(&self, id: Uuid) -> sqlx::Result<AuditEventDto> {
        let entity = self.repo.get_by_id(id).await?;
        Ok(entity.into())
    }

    pub async fn recent(&self, limit: i64) -> sqlx::Result<Vec<AuditEventDto>> {
        let entities = self.repo.list_recent(limit).await?;
        Ok(entities.into_iter().map(Into::into).collect())
    }

    pub async fn by_user(&self, user_id: i64, limit: i64) -> sqlx::Result<Vec<AuditEventDto>> {
        let entities = self.repo.list_by_user(user_id, limit).await?;
        Ok(entities.into_iter().map(Into::into).collect())
    }
}
