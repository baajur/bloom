use crate::{
    config::Config,
    consts::NamespaceType,
    consts::{self, BillingPlan, TwoFaMethod},
    db::DB,
    drivers,
    entities::{Session, User},
    notifications::PAYMENT_ACTION_REQUIRED_EMAIL_TEMPLATE_ID,
    notifications::PAYMENT_FAILED_EMAIL_TEMPLATE,
    notifications::PAYMENT_FAILED_EMAIL_TEMPLATE_ID,
    notifications::REGISTRATION_EMAIL_TEMPLATE,
    notifications::REGISTRATION_EMAIL_TEMPLATE_ID,
    notifications::SIGN_IN_EMAIL_TEMPLATE,
    notifications::{
        EMAIL_CHANGED_EMAIL_TEMPLATE, EMAIL_CHANGED_EMAIL_TEMPLATE_ID, GROUP_INVITATION_EMAIL_TEMPLATE,
        GROUP_INVITATION_EMAIL_TEMPLATE_ID, PAYMENT_ACTION_REQUIRED_EMAIL_TEMPLATE, SIGN_IN_EMAIL_TEMPLATE_ID,
        VERIFY_EMAIL_EMAIL_TEMPLATE, VERIFY_EMAIL_EMAIL_TEMPLATE_ID,
    },
    repository::Repository,
};
use sqlx::types::Uuid;
use std::{collections::HashSet, fmt::Debug, sync::Arc};
use stdx::uuid;

mod accept_group_invitation;
mod cancel_group_invitation;
mod check_namespace_exists;
mod complete_registration;
mod complete_sign_in;
mod complete_two_fa_challenge;
mod complete_two_fa_setup;
mod config;
mod create_group;
mod create_namespace;
mod decline_group_invitation;
mod delete_group;
mod delete_my_account;
mod disable_two_fa;
mod find_group_and_membership;
mod invite_people_in_group;
mod quit_group;
mod register;
mod remove_member_from_group;
mod revoke_session;
mod setup_two_fa;
mod sign_in;
mod update_group_profile;
mod update_my_profile;
mod utils;
mod validators;
mod verify_email;

#[derive(Debug)]
pub struct Service {
    repo: Repository,
    db: DB,
    config: Arc<Config>,
    queue: Arc<dyn drivers::Queue>,
    mailer: Arc<dyn drivers::Mailer>,
    storage: Arc<dyn drivers::Storage>,
    templates: tera::Tera,
    invalid_namespaces: HashSet<String>,
    valid_namespace_alphabet: HashSet<char>,
}

impl Service {
    pub fn new(
        config: Config,
        db: DB,
        queue: Arc<dyn drivers::Queue>,
        mailer: Arc<dyn drivers::Mailer>,
        storage: Arc<dyn drivers::Storage>,
    ) -> Service {
        let mut templates = tera::Tera::default();
        templates
            .add_raw_template(REGISTRATION_EMAIL_TEMPLATE_ID, REGISTRATION_EMAIL_TEMPLATE)
            .expect("kernel: parsing REGISTRATION_EMAIL_TEMPLATE");
        templates
            .add_raw_template(SIGN_IN_EMAIL_TEMPLATE_ID, SIGN_IN_EMAIL_TEMPLATE)
            .expect("kernel: parsing SIGN_IN_EMAIL_TEMPLATE");
        templates
            .add_raw_template(PAYMENT_FAILED_EMAIL_TEMPLATE_ID, PAYMENT_FAILED_EMAIL_TEMPLATE)
            .expect("kernel: parsing PAYMENT_FAILED_EMAIL_TEMPLATE");
        templates
            .add_raw_template(
                PAYMENT_ACTION_REQUIRED_EMAIL_TEMPLATE_ID,
                PAYMENT_ACTION_REQUIRED_EMAIL_TEMPLATE,
            )
            .expect("kernel: parsing PAYMENT_ACTION_REQUIRED_EMAIL_TEMPLATE");
        templates
            .add_raw_template(VERIFY_EMAIL_EMAIL_TEMPLATE_ID, VERIFY_EMAIL_EMAIL_TEMPLATE)
            .expect("kernel: parsing VERIFY_EMAIL_EMAIL_TEMPLATE");
        templates
            .add_raw_template(EMAIL_CHANGED_EMAIL_TEMPLATE_ID, EMAIL_CHANGED_EMAIL_TEMPLATE)
            .expect("kernel: parsing EMAIL_CHANGED_EMAIL_TEMPLATE");
        templates
            .add_raw_template(GROUP_INVITATION_EMAIL_TEMPLATE_ID, GROUP_INVITATION_EMAIL_TEMPLATE)
            .expect("kernel: parsing GROUP_INVITATION_EMAIL_TEMPLATE");

        let repo = Repository::new();

        let invalid_namespaces = consts::INVALID_NAMESPACES
            .iter()
            .map(|namespace| namespace.to_string())
            .collect();

        let valid_namespace_alphabet = consts::NAMESPACE_ALPHABET.chars().collect();

        let config = Arc::new(config);

        Service {
            db,
            repo,
            config,
            queue,
            mailer,
            storage,
            templates,
            invalid_namespaces,
            valid_namespace_alphabet,
        }
    }
}

// #[async_trait::async_trait]
// impl crate::Service for Service {
//     async fn register(&self, actor: Option<&User>, input: RegisterInput) -> Result<PendingUser, Error> {
//         self._register(actor, input).await
//     }
// }

#[derive(Debug, Clone)]
pub enum SignedIn {
    Success { session: Session, user: User },
    TwoFa(TwoFaMethod),
}

/// RegisterInput are the data required to start to register to bloom
#[derive(Debug, Clone)]
pub struct RegisterInput {
    pub email: String,
    pub username: String,
}

/// CompleteRegistrationInput are the data required to complete a bloom registration
#[derive(Debug, Clone)]
pub struct CompleteRegistrationInput {
    pub pending_user_id: Uuid,
    pub code: String,
}

/// CompleteSignInInput are the data required to complete a sign in
#[derive(Debug, Clone)]
pub struct CompleteSignInInput {
    pub pending_session_id: Uuid,
    pub code: String,
}

/// SignInInput are the data required to start a sign in
#[derive(Debug, Clone)]
pub struct SignInInput {
    pub email_or_username: String,
}

#[derive(Debug, Clone)]
pub struct CreateGroupInput {
    pub name: String,
    pub path: String,
    pub description: String,
}

#[derive(Debug, Clone)]
pub struct DeleteGroupInput {
    pub group_id: Uuid,
}

// type GroupNamespace struct {
// 	Group     Group
// 	Namespace Namespace
// }

// type GroupUserNamespace struct {
// 	Group     Group
// 	User      User
// 	Namespace Namespace
// }

#[derive(Debug, Clone)]
pub struct CreateNamespaceInput {
    pub path: String,
    pub namespace_type: NamespaceType,
}

#[derive(Debug, Clone)]
pub struct UpdatePaymentMethodInput {
    pub stripe_id: String,
    pub namespace_id: uuid::Uuid,
}

#[derive(Debug, Clone)]
pub struct ChangeSubscriptionInput {
    pub namespace_id: uuid::Uuid,
    pub plan: BillingPlan,
}

#[derive(Debug, Clone)]
pub struct GetCheckoutSessionInput {
    pub namespace: String,
    pub plan: BillingPlan,
}

#[derive(Debug, Clone)]
pub struct UpdateBillingInformationInput {
    pub namespace: String,
    pub name: String,
    pub email: String,
    pub country_code: String,
    pub city: String,
    pub postal_code: String,
    pub address_line1: String,
    pub address_line2: String,
    pub state: String,
    pub tax_id: Option<String>,
}

#[derive(Debug, Clone)]
pub struct SyncBillingWithProviderInput {
    pub namespace: String,
}

#[derive(Debug, Clone)]
pub struct UpdateMyProfileInput {
    pub name: Option<String>,
    pub email: Option<String>,
    pub username: Option<String>,
    pub description: Option<String>,
    // pub avatar: Option<Vec<u8>>,
}

#[derive(Debug, Clone)]
pub struct VerifyPendingEmailInput {
    pub token: String,
}

#[derive(Debug, Clone)]
pub struct UpdateGroupProfileInput {
    pub group_id: uuid::Uuid,
    pub name: Option<String>,
    pub path: Option<String>,
    pub description: Option<String>,
    // pub avatar: Option<Vec<u8>>,
}

#[derive(Debug, Clone)]
pub struct InvitePeopleInGroupInput {
    pub group_id: uuid::Uuid,
    pub usernames: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct AcceptGroupInvitationInput {
    pub invitation_id: uuid::Uuid,
}

#[derive(Debug, Clone)]
pub struct CancelGroupInvitationInput {
    pub invitation_id: uuid::Uuid,
}

#[derive(Debug, Clone)]
pub struct DeclineGroupInvitationInput {
    pub invitation_id: uuid::Uuid,
}

#[derive(Debug, Clone)]
pub struct RemoveMemberFromGroupInput {
    pub group_id: uuid::Uuid,
    pub username: String,
}

#[derive(Debug, Clone)]
pub struct QuitGroupInput {
    pub group_id: uuid::Uuid,
}

#[derive(Debug, Clone)]
pub struct GetSignedStorageUploadUrlInput {
    pub filesize: u64,
}

#[derive(Debug, Clone)]
pub struct EnableTwoFaInput {
    pub code: String,
}

#[derive(Debug, Clone)]
pub struct DisableTwoFaInput {
    pub code: String,
}

#[derive(Debug, Clone)]
pub struct CompleteTwoFaChallengeInput {
    pub pending_session_id: uuid::Uuid,
    pub code: String,
}

#[derive(Debug, Clone)]
pub struct DeleteMyAccountInput {
    pub two_fa_totp_code: Option<String>,
}

#[derive(Debug, Clone)]
pub struct RevokeSessionInput {
    pub session_id: Uuid,
}

#[derive(Debug, Clone)]
pub struct VerifyEmailInput {
    pub pending_email_id: Uuid,
    pub code: String,
}

#[derive(Debug, Clone)]
pub struct DecodedSessionToken {
    pub session_id: Uuid,
    pub secret: Vec<u8>,
}

// type GroupMember struct {
// 	User
// 	Role GroupRole `db:"role"`
// }

// type SignedStorageUploadUrl struct {
// 	URL    string
// 	TmpKey string
// 	Size   int64
// }

// type NamespaceAndCustomer struct {
// 	Customer
// 	Namespace
// }