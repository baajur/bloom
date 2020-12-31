use super::Repository;
use crate::{db::Queryer, errors::kernel::Error};
use stdx::uuid::Uuid;

impl Repository {
    pub async fn delete_pending_user<'c, C: Queryer<'c>>(&self, db: C, pending_user_id: Uuid) -> Result<(), Error> {
        const QUERY: &str = "DELETE FROM kernel_pending_users WHERE id = $1";

        match sqlx::query(QUERY).bind(pending_user_id).execute(db).await {
            Err(err) => {
                println!("kernel.delete_pending_user: Deleting pending user: {}", &err);
                Err(err.into())
            }
            Ok(_) => Ok(()),
        }
    }
}
