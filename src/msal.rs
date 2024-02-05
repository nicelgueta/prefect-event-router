use azure_identity::client_credentials_flow;
use azure_security_keyvault::{KeyvaultClient, SecretClient};
use azure_identity::DefaultAzureCredential;
use std::sync::Arc;

use crate::interfaces::Error;


fn get_key_vault_client(
    creds: Arc<DefaultAzureCredential>,
    vault_uri: &String
) -> SecretClient {
    let client = KeyvaultClient::new(
        vault_uri,
        creds,
    ).unwrap().secret_client();
    client
}
fn get_key_or_value(env_var_name: &str) -> Result<(bool, String), Error> {
    let raw_value_result = std::env::var(env_var_name);
    match raw_value_result {
        Ok(value) => Ok((false, value)),
        Err(_) => {
            let kv_env_var_name = format!("{}_KV", env_var_name);
            let result = std::env::var(&kv_env_var_name);
            match result {
                Ok(v) => Ok((true, v)),
                Err(_) => Err(Error::InputError(
                    format!(
                        "Instance requires MSAL but unable to determine required env var {} or {}", 
                        &env_var_name, &kv_env_var_name
                    )
                ))
            }
        }
    }
}

pub async fn get_azure_token() -> Result<String, Error> {
    let vault_uri_result = std::env::var("AZURE_KEY_VAULT_URI");
    
    let (client_id_is_key, mut client_id) = get_key_or_value("PREFECT_API_CLIENT_ID")?;
    let (client_secret_is_key, mut client_secret) = get_key_or_value("PREFECT_API_CLIENT_SECRET")?;
    let (tenant_id_is_key, mut tenant_id) = get_key_or_value("PREFECT_API_TENANT_ID")?;
    let (scope_is_key, mut scope) = get_key_or_value("PREFECT_API_SCOPE")?;
    
    let secret_client_result = match vault_uri_result {
        Ok(uri) => {
            let credential = Arc::new(DefaultAzureCredential::default());
            Some(get_key_vault_client(credential, &uri))
        },
        Err(_) => None
    };
    if [client_id_is_key, client_secret_is_key, tenant_id_is_key, scope_is_key].iter().any(|_|true) {
        let secret_client = secret_client_result.expect(
            "Key vault must be specified if any of the auth env vars were marked as being a vault key"
        );
        if client_id_is_key {client_id = secret_client.get(client_id).await.unwrap().value};
        if client_secret_is_key {client_secret = secret_client.get(client_secret).await.unwrap().value};
        if tenant_id_is_key {tenant_id = secret_client.get(tenant_id).await.unwrap().value};
        if scope_is_key {scope = secret_client.get(scope).await.unwrap().value};
    }

    let http_client = azure_core::new_http_client();

    let token = client_credentials_flow::perform(
        http_client.clone(),
        &client_id,
        &client_secret,
        &[&scope],
        &tenant_id,
    ).await;
    Ok(String::from(token.unwrap().access_token.secret()))
}