use cqrs_es::persist::PersistedEventStore;
use cqrs_es::{Aggregate, CqrsFramework, Query};
use std::sync::Arc;

use crate::{MysqlCqrs, MysqlEventRepository};
use sqlx::mysql::MySqlPoolOptions;
use sqlx::{MySql, Pool};

/// A convenience building a simple connection pool for MySql database.
pub async fn default_mysql_pool(connection_string: &str) -> Pool<MySql> {
    MySqlPoolOptions::new()
        .max_connections(10)
        .connect(connection_string)
        .await
        .expect("unable to connect to database")
}

/// A convenience function for creating a CqrsFramework from a database connection pool
/// and queries.
pub fn mysql_cqrs<A>(pool: Pool<MySql>, query_processor: Vec<Arc<dyn Query<A>>>) -> MysqlCqrs<A>
where
    A: Aggregate,
{
    let repo = MysqlEventRepository::new(pool);
    let store = PersistedEventStore::new_event_store(repo);
    CqrsFramework::new(store, query_processor)
}

/// A convenience function for creating a CqrsFramework using a snapshot store.
pub fn mysql_snapshot_cqrs<A>(
    pool: Pool<MySql>,
    query_processor: Vec<Arc<dyn Query<A>>>,
    snapshot_size: usize,
) -> MysqlCqrs<A>
where
    A: Aggregate,
{
    let repo = MysqlEventRepository::new(pool);
    let store = PersistedEventStore::new_snapshot_store(repo, snapshot_size);
    CqrsFramework::new(store, query_processor)
}

/// A convenience function for creating a CqrsFramework using an aggregate store.
pub fn mysql_aggregate_cqrs<A>(
    pool: Pool<MySql>,
    query_processor: Vec<Arc<dyn Query<A>>>,
) -> MysqlCqrs<A>
where
    A: Aggregate,
{
    let repo = MysqlEventRepository::new(pool);
    let store = PersistedEventStore::new_aggregate_store(repo);
    CqrsFramework::new(store, query_processor)
}

#[cfg(test)]
mod test {
    use crate::testing::tests::{
        TestAggregate, TestQueryRepository, TestView, TEST_CONNECTION_STRING,
    };
    use crate::{default_mysql_pool, mysql_cqrs, MysqlViewRepository};
    use std::sync::Arc;

    #[tokio::test]
    async fn test_valid_cqrs_framework() {
        let pool = default_mysql_pool(TEST_CONNECTION_STRING).await;
        let repo = MysqlViewRepository::<TestView, TestAggregate>::new("test_query", pool.clone());
        let query = TestQueryRepository::new(repo);
        let _ps = mysql_cqrs(pool, vec![Arc::new(query)]);
    }
}
