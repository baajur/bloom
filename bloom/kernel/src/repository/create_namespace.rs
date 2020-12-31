use super::Repository;
use crate::{db, entities, errors::kernel::Error};

impl Repository {
    pub async fn create_namespace<'c, C: db::Queryer<'c>>(
        &self,
        db: C,
        namespace: &entities::Namespace,
    ) -> Result<(), Error> {
        const QUERY: &str = "INSERT INTO kernel_namespaces
            (id, created_at, updated_at, path, type, parent_id)
            VALUES ($1, $2, $3, $4, $5, $6)";

        match sqlx::query(QUERY)
            .bind(namespace.id)
            .bind(namespace.created_at)
            .bind(namespace.updated_at)
            .bind(&namespace.path)
            .bind(namespace.r#type)
            .bind(namespace.parent_id)
            .execute(db)
            .await
        {
            Err(err) => {
                println!("kernel.create_namespace: Inserting namespace: {}", &err);
                Err(err.into())
            }
            Ok(_) => Ok(()),
        }
    }
}
