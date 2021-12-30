use std::marker::PhantomData;

use async_trait::async_trait;
use cqrs_es::{Aggregate, View};
use persist_es::{PersistenceError, QueryContext, ViewRepository};
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
