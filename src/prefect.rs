use serde::{Deserialize, Serialize};
use crate::interfaces::Error;
use crate::config;
use std::sync::Arc;

#[cfg(feature = "azure_storage_queues")]
use crate::msal;

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Like {
    like_: String
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Filter {
    name: Like
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct DeploymentBody {
    flows: Filter,
    deployments: Filter
}

async fn get_deployment_id(
    prefect_uri: &String,
    token: Option<&String>, flow_name: &String, deployment_name: &String
) -> Result<String, Error> {
    let flow_like = Like {like_:flow_name.clone()};
    let deployment_like = Like {like_:deployment_name.clone()};
    let flow_filter = Filter {name: flow_like};
    let deployment_filter = Filter {name: deployment_like};
    let body = DeploymentBody{
        flows: flow_filter, deployments: deployment_filter
    };
    let mut req_builder = reqwest::Client::new()
        .post(format!("{}/deployments/filter", prefect_uri));
    if let Some(token_value) = token {
        req_builder = req_builder.header("Authorization", format!("Bearer {}", token_value))
    }
    let json_res = req_builder
        .json(&body)
        .send()
        .await.unwrap()
        .json()
        .await;
    let res: serde_json::Value = match json_res {
        Ok(v) => Ok(v),
        Err(e) => Err(Error::PrefectApiError(
            format!("Deployment ID call does not return JSON. Got {}. Are credentials set correctly?", e.to_string())
        ))
    }?;
    let deployment_id = match res[0]["id"].as_str() {
        Some(id) => id,
        None => return Err(
            Error::PrefectApiError(
                "Unable to find deployment ID for the given flow/deployment".to_string()
            )
        )
    };
    Ok(deployment_id.to_string())
}

async fn get_token(settings_ptr: &Arc<config::Settings>) -> Result<Option<String>, Error> {
    let mut token: Option<String> = None;

    #[cfg(feature = "azure_storage_queues")]
    if let Some(v) = settings_ptr.prefect_use_msal_auth {
        if v {
            let (cid, csec, ten, scop) = settings_ptr.get_azure_credentials().expect(
                "Unable to source credentials from the shared settings"
            );
            token = Some(msal::get_azure_token(cid, csec, ten, scop).await?)
        } else {token = None}
    };
    if let None = token {
        token = match std::env::var("PREFECT_API_KEY") {
            Ok(v) => Some(v), Err(_) => None
        }
    };
    Ok(token)
}
pub async fn trigger_prefect_deployment(
    flow_name: &String,
    deployment_name: &String,
    flow_parameters: &Option<serde_json::Value>,
    settings_ptr: &Arc<config::Settings>
) -> Result<String, Error> {
    let prefect_uri = std::env::var("PREFECT_API_URI").expect(
        "Env var PREFECT_API_URI is required for this application to run"
    );
    let token = get_token(settings_ptr).await?;
    let deployment_id = get_deployment_id(
        &prefect_uri, token.as_ref(), flow_name, deployment_name
    ).await?;
    let uri = format!("{}/deployments/{}/create_flow_run", &prefect_uri, &deployment_id);
    let mut req_builder = reqwest::Client::new()
        .post(uri);
    if let Some(token_value) = token {
        req_builder = req_builder.header("Authorization", format!("Bearer {}", token_value));
    };
    let body = match flow_parameters {
        Some(params) => serde_json::json!({"parameters": params}),
        None => serde_json::json!({})
    };
    req_builder = req_builder.json(&body);
    let res: serde_json::Value = req_builder  
        .send()
        .await.unwrap()
        .json().await.unwrap();
    Ok(
        String::from(
            res["name"]
            .as_str()
            .expect("Unable to get name from the returned json")
        )
    )
}