use std::marker::PhantomData;

use async_trait::async_trait;
use cqrs_es::persist::{PersistenceError, QueryContext, ViewRepository};
use cqrs_es::{Aggregate, View};
use sqlx::mysql::MySqlRow;
use sqlx::{MySql, Pool, Row};

use crate::error::MysqlAggregateError;

/// A mysql backed query repository for use in backing a `GenericQuery`.
pub struct MysqlViewRepository<V, A> {
    insert_sql: String,
    update_sql: String,
    select_sql: String,
    pool: Pool<MySql>,
    _phantom: PhantomData<(V, A)>,
}

impl<V, A> MysqlViewRepository<V, A>
where
    V: View<A>,
    A: Aggregate,
{
    /// Creates a new `MysqlViewRepository` that will store serialized views in a MySql table named
    /// identically to the `query_name` value provided. This table should be created by the user
    /// before using this query repository (see `/db/init.sql` sql initialization file).
    pub fn new(query_name: &str, pool: Pool<MySql>) -> Self {
        let insert_sql = format!(
            "INSERT INTO {} (payload, version, query_instance_id) VALUES ( ?, ?, ? )",
            query_name
        );
        let update_sql = format!(
            "UPDATE {} SET payload= ? , version= ? WHERE query_instance_id= ?",
            query_name
        );
        let select_sql = format!(
            "SELECT version,payload FROM {} WHERE query_instance_id= ?",
            query_name
        );
        Self {
            insert_sql,
            update_sql,
            select_sql,
            pool,
            _phantom: Default::default(),
        }
    }
}

#[async_trait]
impl<V, A> ViewRepository<V, A> for MysqlViewRepository<V, A>
where
    V: View<A>,
    A: Aggregate,
{
    async fn load(
        &self,
        query_instance_id: &str,
    ) -> Result<Option<(V, QueryContext)>, PersistenceError> {
        let row: Option<MySqlRow> = sqlx::query(&self.select_sql)
            .bind(&query_instance_id)
            .fetch_optional(&self.pool)
            .await
            .map_err(MysqlAggregateError::from)?;
        match row {
            None => Ok(None),
            Some(row) => {
                let version = row.get("version");
                let view = serde_json::from_value(row.get("payload"))
                    .map_err(MysqlAggregateError::from)?;
                let view_context = QueryContext::new(query_instance_id.to_string(), version);
                Ok(Some((view, view_context)))
            }
        }
    }

    async fn update_view(&self, view: V, context: QueryContext) -> Result<(), PersistenceError> {
        let sql = match context.version {
            0 => &self.insert_sql,
            _ => &self.update_sql,
        };
        let version = context.version + 1;
        let payload = serde_json::to_value(&view).map_err(MysqlAggregateError::from)?;
        sqlx::query(sql.as_str())
            .bind(payload)
            .bind(&version)
            .bind(context.view_instance_id)
            .execute(&self.pool)
            .await
            .map_err(MysqlAggregateError::from)?;
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use crate::testing::tests::{
        Created, TestAggregate, TestEvent, TestView, TEST_CONNECTION_STRING,
    };
    use crate::{default_mysql_pool, MysqlViewRepository};
    use cqrs_es::persist::{QueryContext, ViewRepository};

    #[tokio::test]
    async fn test_valid_view_repository() {
        let pool = default_mysql_pool(TEST_CONNECTION_STRING).await;
        let repo = MysqlViewRepository::<TestView, TestAggregate>::new("test_query", pool.clone());
        let test_view_id = uuid::Uuid::new_v4().to_string();

        let view = TestView {
            events: vec![TestEvent::Created(Created {
                id: "just a test event for this view".to_string(),
            })],
        };
        repo.update_view(view.clone(), QueryContext::new(test_view_id.to_string(), 0))
            .await
            .unwrap();
        let (found, context) = repo.load(&test_view_id).await.unwrap().unwrap();
        assert_eq!(found, view);

        let updated_view = TestView {
            events: vec![TestEvent::Created(Created {
                id: "a totally different view".to_string(),
            })],
        };
        repo.update_view(updated_view.clone(), context)
            .await
            .unwrap();
        let found_option = repo.load(&test_view_id).await.unwrap();
        let found = found_option.unwrap().0;

        assert_eq!(found, updated_view);
    }
}
