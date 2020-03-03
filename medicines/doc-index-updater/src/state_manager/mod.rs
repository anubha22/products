pub use self::redis::get_client;
use self::redis::{get_from_redis, set_in_redis, MyRedisError};
use crate::models::{JobStatus, JobStatusResponse};
use ::redis::Client;
use uuid::Uuid;
use warp::{http::StatusCode, reply::Json, Filter, Rejection, Reply};
mod redis;

#[derive(Clone, Debug)]
pub struct StateManager {
    pub client: Client,
}

impl StateManager {
    pub fn new(client: Client) -> Self {
        StateManager { client }
    }

    pub async fn get_status(&self, id: Uuid) -> Result<JobStatusResponse, MyRedisError> {
        let status = get_from_redis(self.client.clone(), id).await?;

        Ok(JobStatusResponse { id, status })
    }

    pub async fn set_status(
        &self,
        id: Uuid,
        status: JobStatus,
    ) -> Result<JobStatusResponse, MyRedisError> {
        let status = set_in_redis(self.client.clone(), id, status).await?;

        Ok(JobStatusResponse { id, status })
    }
}

pub fn get_job_status(
    state_manager: StateManager,
) -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
    warp::path!("jobs" / Uuid)
        .and(warp::get())
        .and(with_state(state_manager))
        .and_then(get_status_handler)
}

fn handle_known_errors(id: Uuid, e: MyRedisError) -> Result<JobStatusResponse, MyRedisError> {
    tracing::error!("{}", e);
    if let MyRedisError::IncompatibleType(_) = e {
        Ok(JobStatusResponse {
            id,
            status: JobStatus::NotFound,
        })
    } else {
        Err(e)
    }
}

fn to_response_with_status(response: JobStatusResponse) -> impl Reply {
    let json = warp::reply::json(&response);
    match response.status {
        JobStatus::NotFound => warp::reply::with_status(json, StatusCode::NOT_FOUND),
        _ => warp::reply::with_status(json, StatusCode::OK),
    }
}

#[tracing::instrument]
async fn get_status_handler(id: Uuid, mgr: StateManager) -> Result<impl Reply, Rejection> {
    mgr.get_status(id)
        .await
        .or_else(|e| handle_known_errors(id, e))
        .map_err(Into::into)
        .map(to_response_with_status)
}

pub fn set_job_status(
    state_manager: StateManager,
) -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
    warp::path!("jobs" / Uuid / JobStatus)
        .and(warp::post())
        .and(with_state(state_manager))
        .and_then(set_status_handler)
}

#[tracing::instrument]
async fn set_status_handler(
    id: Uuid,
    status: JobStatus,
    mgr: StateManager,
) -> Result<Json, Rejection> {
    mgr.set_status(id, status)
        .await
        .or_else(|e: MyRedisError| {
            tracing::error!("{}", e);
            Err(e)
        })
        .map_err(Into::into)
        .map(|r| warp::reply::json(&r))
}

pub fn with_state(
    mgr: StateManager,
) -> impl Filter<Extract = (StateManager,), Error = std::convert::Infallible> + Clone {
    warp::any().map(move || mgr.clone())
}
