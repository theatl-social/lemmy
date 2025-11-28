use crate::{
  activity_lists::SharedInboxActivities,
  fetcher::user_or_community::UserOrCommunity,
  protocol::objects::tombstone::Tombstone,
  FEDERATION_CONTEXT,
};
use activitypub_federation::{
  actix_web::inbox::receive_activity,
  config::Data,
  protocol::context::WithContext,
  FEDERATION_CONTENT_TYPE,
};
use actix_web::{web, web::Bytes, HttpRequest, HttpResponse};
use http::{header::LOCATION, StatusCode};
use lemmy_api_common::context::LemmyContext;
use lemmy_db_schema::{
  newtypes::DbUrl,
  source::{activity::SentActivity, community::Community},
  CommunityVisibility,
};
use lemmy_utils::error::{LemmyErrorType, LemmyResult};
use serde::{Deserialize, Serialize};
use std::{ops::Deref, time::Duration};
use tokio::time::timeout;
use tracing::warn;
use url::Url;

mod comment;
mod community;
mod person;
mod post;
pub mod routes;
pub mod site;

/// Helper struct to extract basic activity info for debugging purposes.
/// This is used to log meaningful information when an InboxTimeout occurs.
#[derive(Deserialize, Debug, Default)]
struct ActivityDebugInfo {
  /// The activity ID (e.g., "https://example.com/activities/create/123")
  id: Option<String>,
  /// The activity type (e.g., "Create", "Announce", "Follow")
  #[serde(rename = "type")]
  activity_type: Option<String>,
  /// The actor who initiated the activity (e.g., "https://example.com/u/user")
  actor: Option<String>,
}

pub async fn shared_inbox(
  request: HttpRequest,
  body: Bytes,
  data: Data<LemmyContext>,
) -> LemmyResult<HttpResponse> {
  // Clone the body for potential debug logging on timeout.
  // Bytes is reference-counted, so this is cheap.
  let body_for_debug = body.clone();

  let receive_fut =
    receive_activity::<SharedInboxActivities, UserOrCommunity, LemmyContext>(request, body, &data);
  // Use configurable timeout for processing incoming activities. This allows administrators
  // to tune the timeout based on their infrastructure (database latency, network conditions, etc.).
  // If processing takes longer than this timeout, we return early with an InboxTimeout error
  // to avoid being marked as dead by the sender.
  let timeout_secs = data.settings().federation.incoming_activity_timeout;
  let activity_timeout = Duration::from_secs(timeout_secs);
  timeout(activity_timeout, receive_fut).await.map_err(|_| {
    // Only parse the activity body when a timeout occurs to avoid overhead on successful requests.
    match serde_json::from_slice::<ActivityDebugInfo>(&body_for_debug) {
      Ok(debug_info) => {
        warn!(
          activity_id = ?debug_info.id,
          activity_type = ?debug_info.activity_type,
          actor = ?debug_info.actor,
          timeout_secs = timeout_secs,
          "Inbox activity processing timed out after {} seconds. Consider increasing federation.incoming_activity_timeout if this happens frequently.",
          timeout_secs
        );
      }
      Err(parse_error) => {
        warn!(
          timeout_secs = timeout_secs,
          parse_error = %parse_error,
          "Inbox activity processing timed out after {} seconds, and activity body could not be parsed for debugging. Consider increasing federation.incoming_activity_timeout if this happens frequently.",
          timeout_secs
        );
      }
    }
    LemmyErrorType::InboxTimeout
  })?
}

/// Convert the data to json and turn it into an HTTP Response with the correct ActivityPub
/// headers.
///
/// actix-web doesn't allow pretty-print for json so we need to do this manually.
fn create_apub_response<T>(data: &T) -> LemmyResult<HttpResponse>
where
  T: Serialize,
{
  let json = serde_json::to_string_pretty(&WithContext::new(data, FEDERATION_CONTEXT.clone()))?;

  Ok(
    HttpResponse::Ok()
      .content_type(FEDERATION_CONTENT_TYPE)
      .body(json),
  )
}

fn create_apub_tombstone_response<T: Into<Url>>(id: T) -> LemmyResult<HttpResponse> {
  let tombstone = Tombstone::new(id.into());
  let json = serde_json::to_string_pretty(&WithContext::new(
    tombstone,
    FEDERATION_CONTEXT.deref().clone(),
  ))?;

  Ok(
    HttpResponse::Gone()
      .content_type(FEDERATION_CONTENT_TYPE)
      .status(StatusCode::GONE)
      .body(json),
  )
}

fn redirect_remote_object(url: &DbUrl) -> HttpResponse {
  let mut res = HttpResponse::PermanentRedirect();
  res.insert_header((LOCATION, url.as_str()));
  res.finish()
}

#[derive(Deserialize)]
pub struct ActivityQuery {
  type_: String,
  id: String,
}

/// Return the ActivityPub json representation of a local activity over HTTP.
#[tracing::instrument(skip_all)]
pub(crate) async fn get_activity(
  info: web::Path<ActivityQuery>,
  context: web::Data<LemmyContext>,
) -> LemmyResult<HttpResponse> {
  let settings = context.settings();
  let activity_id = Url::parse(&format!(
    "{}/activities/{}/{}",
    settings.get_protocol_and_hostname(),
    info.type_,
    info.id
  ))?
  .into();
  let activity = SentActivity::read_from_apub_id(&mut context.pool(), &activity_id)
    .await?
    .ok_or(LemmyErrorType::CouldntFindActivity)?;

  let sensitive = activity.sensitive;
  if sensitive {
    Ok(HttpResponse::Forbidden().finish())
  } else {
    create_apub_response(&activity.data)
  }
}

/// Ensure that the community is public and not removed/deleted.
fn check_community_public(community: &Community) -> LemmyResult<()> {
  if community.deleted || community.removed {
    Err(LemmyErrorType::Deleted)?
  }
  if community.visibility != CommunityVisibility::Public {
    return Err(LemmyErrorType::CouldntFindCommunity.into());
  }
  Ok(())
}
