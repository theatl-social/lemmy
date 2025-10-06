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

#[cfg(test)]
mod tests {
  use super::*;
  use actix_web::test::TestRequest;
  use lemmy_api_common::context::LemmyContext;
  use lemmy_api_common::person::PrivilegedRegister;
  use lemmy_api_crud::user::create::privileged_register;
  use lemmy_db_schema::{
    source::{
      instance::Instance,
      local_site::{LocalSite, LocalSiteInsertForm},
      secret::Secret,
      site::{Site, SiteInsertForm},
    },
    traits::Crud,
    utils::build_db_pool_for_tests,
  };
  use lemmy_utils::rate_limit::RateLimitCell;
  use reqwest::Client;
  use reqwest_middleware::ClientBuilder;
  use serial_test::serial;

  async fn setup_test_context(secret: Option<String>) -> LemmyContext {
    let pool = build_db_pool_for_tests().await;
    let pool_ref = &mut (&pool).into();

    let db_secret = Secret::init(pool_ref).await.unwrap().unwrap();

    let mut settings = lemmy_utils::settings::SETTINGS.clone();
    if let Some(s) = secret {
      settings.private_api_secret = Some(s);
    }

    let context = LemmyContext {
      pool: pool.clone(),
      client: ClientBuilder::new(Client::default()).build(),
      secret: db_secret,
      settings: std::sync::Arc::new(settings),
      rate_limit_cell: RateLimitCell::with_test_config(),
    };

    // Setup site
    let instance = Instance::read_or_create(pool_ref, "test_instance".to_string())
      .await
      .unwrap();
    let site_form = SiteInsertForm::new("test site".to_string(), instance.id);
    let site = Site::create(pool_ref, &site_form).await.unwrap();
    let local_site_form = LocalSiteInsertForm::builder().site_id(site.id).build();
    LocalSite::create(pool_ref, &local_site_form).await.unwrap();

    context
  }

  #[tokio::test]
  #[serial]
  async fn test_check_email_not_exists() {
    let secret = "test_secret_123".to_string();
    let context = setup_test_context(Some(secret.clone())).await;

    let data = Json(CheckEmail {
      email: "nonexistent@example.com".to_string().into(),
      api_secret: secret.into(),
    });

    let result = check_email_registered(data, context.clone().into()).await;
    assert!(result.is_ok());

    let response = result.unwrap().into_inner();
    assert!(!response.exists);
    assert!(response.person_id.is_none());
    assert!(response.username.is_none());
  }

  #[tokio::test]
  #[serial]
  async fn test_check_email_exists() {
    let secret = "test_secret_123".to_string();
    let context = setup_test_context(Some(secret.clone())).await;

    let email = "exists@example.com".to_string();

    // First create a user
    let req = TestRequest::default().to_http_request();
    let register_data = Json(PrivilegedRegister {
      username: "existinguser".to_string(),
      password: "ValidPassword123!".to_string().into(),
      email: email.clone().into(),
      api_secret: secret.clone().into(),
    });
    privileged_register(register_data, req, context.clone().into())
      .await
      .unwrap();

    // Now check if email exists
    let check_data = Json(CheckEmail {
      email: email.into(),
      api_secret: secret.into(),
    });

    let result = check_email_registered(check_data, context.clone().into()).await;
    assert!(result.is_ok());

    let response = result.unwrap().into_inner();
    assert!(response.exists);
    assert!(response.person_id.is_some());
    assert_eq!(response.username, Some("existinguser".to_string()));
  }

  #[tokio::test]
  #[serial]
  async fn test_check_email_invalid_secret() {
    let secret = "correct_secret".to_string();
    let context = setup_test_context(Some(secret)).await;

    let data = Json(CheckEmail {
      email: "test@example.com".to_string().into(),
      api_secret: "wrong_secret".to_string().into(),
    });

    let result = check_email_registered(data, context.clone().into()).await;
    assert!(result.is_err());
    assert_eq!(
      result.unwrap_err().error_type,
      LemmyErrorType::IncorrectLogin
    );
  }

  #[tokio::test]
  #[serial]
  async fn test_check_email_no_secret_configured() {
    let context = setup_test_context(None).await;

    let data = Json(CheckEmail {
      email: "test@example.com".to_string().into(),
      api_secret: "any_secret".to_string().into(),
    });

    let result = check_email_registered(data, context.clone().into()).await;
    assert!(result.is_err());
    assert_eq!(
      result.unwrap_err().error_type,
      LemmyErrorType::PrivateApiSecretNotConfigured
    );
  }

  #[tokio::test]
  #[serial]
  async fn test_check_email_case_insensitive() {
    let secret = "test_secret_123".to_string();
    let context = setup_test_context(Some(secret.clone())).await;

    // Create user with lowercase email
    let req = TestRequest::default().to_http_request();
    let register_data = Json(PrivilegedRegister {
      username: "caseuser".to_string(),
      password: "ValidPassword123!".to_string().into(),
      email: "case@example.com".to_string().into(),
      api_secret: secret.clone().into(),
    });
    privileged_register(register_data, req, context.clone().into())
      .await
      .unwrap();

    // Check with uppercase email
    let check_data = Json(CheckEmail {
      email: "CASE@EXAMPLE.COM".to_string().into(),
      api_secret: secret.into(),
    });

    let result = check_email_registered(check_data, context.clone().into()).await;
    assert!(result.is_ok());

    let response = result.unwrap().into_inner();
    assert!(response.exists);
    assert_eq!(response.username, Some("caseuser".to_string()));
  }
}
