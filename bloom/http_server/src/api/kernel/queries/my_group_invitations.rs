use crate::{api::kernel::model::GroupInvitation, ServerContext};
use actix_web::web;
use kernel::{http::api, Actor};
use std::sync::Arc;

pub async fn my_group_invitations(
    ctx: web::Data<Arc<ServerContext>>,
    actor: Actor,
) -> Result<api::Response<Vec<GroupInvitation>>, kernel::Error> {
    let invitations = ctx.kernel_service.find_my_group_invitations(actor).await?;

    Ok(api::Response::ok(invitations.into_iter().map(Into::into).collect()))
}
