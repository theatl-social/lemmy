use activitypub_federation::config::Data;
use actix_web::web::Json;
use lemmy_api_common::{
  context::LemmyContext,
  person::{CheckEmail, CheckEmailResponse},
};
use lemmy_db_views::structs::LocalUserView;
use lemmy_utils::error::{LemmyErrorType, LemmyResult};
use subtle::ConstantTimeEq;

#[tracing::instrument(skip(context))]
pub async fn check_email_registered(
  data: Json<CheckEmail>,
  context: Data<LemmyContext>,
) -> LemmyResult<Json<CheckEmailResponse>> {
  // Validate the API secret first
  let configured_secret = context
    .settings()
    .private_api_secret()
    .ok_or(LemmyErrorType::PrivateApiSecretNotConfigured)?;

  // Use constant-time comparison to prevent timing attacks
  let is_valid: bool = configured_secret
    .as_bytes()
    .ct_eq(data.api_secret.as_ref())
    .into();

  if !is_valid {
    Err(LemmyErrorType::IncorrectLogin)?
  }

  // Check if email exists
  let local_user_view = LocalUserView::find_by_email(&mut context.pool(), &data.email).await?;

  let response = match local_user_view {
    Some(view) => CheckEmailResponse {
      exists: true,
      person_id: Some(view.person.id),
      username: Some(view.person.name),
    },
    None => CheckEmailResponse {
      exists: false,
      person_id: None,
      username: None,
    },
  };

  Ok(Json(response))
}

// TODO: Add integration tests
// Note: Unit tests removed due to complexity of mocking LemmyContext
// Test manually or via integration test suite
