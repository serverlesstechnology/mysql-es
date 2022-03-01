use crate::MysqlEventRepository;
use cqrs_es::persist::{PersistedEventStore, PersistedSnapshotStore};
use cqrs_es::CqrsFramework;

/// A convenience type for a CqrsFramework backed by
/// [MysqlStore](struct.MysqlStore.html).
pub type MysqlCqrs<A> = CqrsFramework<A, PersistedEventStore<MysqlEventRepository, A>>;

/// A convenience type for a CqrsFramework backed by
/// [MysqlSnapshotStore](struct.MysqlSnapshotStore.html).
pub type MysqlSnapshotCqrs<A> = CqrsFramework<A, PersistedSnapshotStore<MysqlEventRepository, A>>;
