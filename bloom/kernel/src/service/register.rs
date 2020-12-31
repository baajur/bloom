use super::{RegisterInput, Service};
use crate::{
    consts,
    entities::{PendingUser, User},
    errors::kernel::Error,
};
use stdx::{
    chrono, crypto,
    rand::{thread_rng, Rng},
};
use stdx::{sync::threadpool::spawn_blocking, ulid::Ulid};
use tokio::time::delay_for;

impl Service {
    pub async fn register(&self, actor: Option<User>, input: RegisterInput) -> Result<PendingUser, crate::Error> {
        if actor.is_some() {
            return Err(Error::MustNotBeAuthenticated.into());
        }

        // sleep to prevent spam and bruteforce
        let sleep = thread_rng().gen_range(consts::SLEEP_MIN..consts::SLEEP_MAX);
        delay_for(sleep).await;

        // clean and validate data
        let email = input.email.trim().to_lowercase();
        self.validate_email(&email, true)?;
        let username = input.username.trim().to_lowercase();
        self.validate_username(&username)?;

        let mut tx = self.db.begin().await?;

        let find_existing_user_res = self.repo.find_user_by_email(&mut tx, &email).await;
        match find_existing_user_res {
            Ok(_) => Err(Error::EmailAlreadyExists),
            Err(Error::UserNotFound) => Ok(()),
            Err(err) => Err(err),
        }?;

        let namespace_exists = self.check_namespace_exists(&mut tx, &username).await?;
        if namespace_exists {
            return Err(Error::UsernameAlreadyExists.into());
        }

        // create new pending user
        let now = chrono::Utc::now();
        let (code, code_hash) = spawn_blocking(|| {
            let code = crypto::rand::alphabet(consts::CODE_ALPHABET, consts::REGISTER_CODE_LENGTH);
            // 	errMessage := "kernel.Register: generating code"
            // 	logger.Error(errMessage, log.Err("error", err))

            let code_hash = crypto::hash_password(&code);
            // 	errMessage := "kernel.Register: hashing code"
            // 	logger.Error(errMessage, log.Err("error", err))

            (code, code_hash)
        })
        .await?;

        let pending_user = PendingUser {
            id: Ulid::new().into(),
            created_at: now,
            updated_at: now,
            username,
            email,
            failed_attempts: 0,
            code_hash,
        };
        self.repo.create_pending_user(&mut tx, &pending_user).await?;

        tx.commit().await?;

        let job = crate::domain::messages::Message::KenrnelSendRegisterEmail {
            email: pending_user.email.clone(),
            username: pending_user.username.clone(),
            code,
        };
        let _ = self.queue.push(job, None).await; // TODO: log error?

        Ok(pending_user)
    }
}
