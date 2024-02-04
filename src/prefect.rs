use serde::{Deserialize, Serialize};

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
) -> Result<String, reqwest::Error> {
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
    let res: serde_json::Value = req_builder
        .json(&body)
        .send()
        .await?
        .json()
        .await?;
    let deployment_id: String = String::from(res[0]["id"].as_str().expect(
        "No id found in the returned deployment"
    ));
    Ok(deployment_id.to_string())
}

pub async fn trigger_prefect_deployment(
    flow_name: &String,
    deployment_name: &String,
    flow_parameters: &Option<serde_json::Value>
) -> Result<String, reqwest::Error> {
    let prefect_uri = std::env::var("PREFECT_API_URI").expect(
        "Env var PREFECT_API_URI is required"
    );
    let token = match std::env::var("PREFECT_API_KEY") {
        Ok(v) => Some(v), Err(_) => None
    };
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
        .await?
        .json().await?;
    Ok(
        String::from(
            res["name"]
            .as_str()
            .expect("Unable to get name from the returned json")
        )
    )
}