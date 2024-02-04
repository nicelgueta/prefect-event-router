mod prefect;
mod interfaces;
mod config;
mod publishers;

use azure_identity::DefaultAzureCredential;
use azure_storage::ErrorKind;
use std::fs::File;
use std::io::Read;
use std::sync::Arc;
use tokio::task::JoinSet;
use publishers::PublisherType;
use interfaces::{Publisher, QMessage, RawMessage};

async fn thread_loop<C>(
    publisher: impl Publisher<C>,
    credential: Option<C>,
) -> azure_core::Result<()> {
    // initialise the connection
    let loop_init = publisher.init(credential);
    let loop_name = publisher.repr();
    
    loop {

        for message in publisher.get_messages(&loop_init).await {
            println!("{}: Found message", &publisher.repr());
            let q_message: QMessage = match serde_json::from_str(&message.get_content_str()) {
                Ok(value) => value,
                Err(error) => {
                    println!("{}: Unable to construct valid Qmessage from content: {:?}", &loop_name, error);
                    continue
                }
            };
            // get the message type config
            let message_type = q_message.typ();
            let flow_dep_result = publisher.get_flow_deployment(&message_type);
            let (flow_name, dep_name) = match flow_dep_result {
                Ok(value) => value,
                Err(inp_error) => {
                    println!("{}: {}", &loop_name, inp_error.as_str());
                    continue
                }
            };
            let flow_parameters = q_message.get_flow_parameters();
            // trigger prefect deployment
            let trigger_result = prefect::trigger_prefect_deployment(
                &flow_name, &dep_name, flow_parameters
            ).await;
            match trigger_result {
                Ok(flow_run_name) => {
                    println!("{}: Successfully triggered {}/{}: {}", &loop_name, &flow_name, &dep_name, &flow_run_name);
                    if let Some(params) = flow_parameters {
                        println!("{}: with parameters {}", &loop_name, params)
                    }
                    publisher.task_done(&loop_init, message).await;
                },
                Err(_) => panic!("{}: Failed to execute prefect deployment trigger", &loop_name)
            };
        }
    }
}

fn load_config_file_str(filepath: String) -> String {
    let mut data = String::new();
    let mut file = File::open(&filepath).expect(
        "file does not exist"
    );
    file.read_to_string(&mut data).expect("Failed to read file");
    data
}
fn config_from_str(config_str: String) -> config::ConfigFile {
    serde_json::from_str(&config_str).expect(
        "File does nto appear to be a valid config file"
    )
}

#[tokio::main]
async fn main() -> azure_core::Result<()> {
    let args: Vec<String> = std::env::args().collect();
    if args.len() > 2 {
        println!("Must only provide 1 CLI arguments [config path]");
        return Err(azure_core::Error::from(ErrorKind::Other));
    }
    let file_path = args[1].clone();
    drop(args);
    let config_str = load_config_file_str(file_path);
    let config: config::ConfigFile = config_from_str(config_str);
    
    // to save duplication we want all threads using azure storage to 
    // share the same auth credential contained within an arc smart pointer 
    // so we check to see if one needs to be
    // created that they all can share.
    let mut uses_azure_storage = false;
    for pt in config.iter() {
        match pt {
            PublisherType::AzureStorageQueue(_) => {
                if ! uses_azure_storage {
                    uses_azure_storage = true
                }
            },
            PublisherType::StdInput(_) => ()
        }
    };
    let azure_token_credential = if uses_azure_storage {
        Some(Arc::new(DefaultAzureCredential::default()))
    } else {
        None
    };
    println!("Event Handler - main | Preparing queue listener service...");

    // create an async thread for each queue
    let mut spawn_set = JoinSet::new();
    let config_iter = config.iter().cloned();
    for pt in config_iter {
        let pub_name = match pt {
            PublisherType::AzureStorageQueue(queue_config) => {
                let token_ptr = azure_token_credential.clone().unwrap();
                let qc_clone = queue_config.clone();
                spawn_set.spawn( async move{
                    thread_loop(qc_clone, Some(token_ptr.clone())).await.unwrap();
                });
                queue_config.repr()
            },
            PublisherType::StdInput(pub_config) => {
                let pcc = pub_config.clone();
                spawn_set.spawn( async move{
                    thread_loop(pcc, None).await.unwrap();
                });
                pub_config.repr()
            }
        };
        println!("Event Handler - main | Spawning thread to listen to: {}", &pub_name);
    };
    while let Some(res) = spawn_set.join_next().await {
        // let out = res.unwrap();
        res.unwrap();
    }
    Ok(())
    
}


#[cfg(test)]
mod tests {
    
}
