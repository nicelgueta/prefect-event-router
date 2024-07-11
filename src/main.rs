mod prefect;
mod interfaces;
mod config;
mod publishers;

#[cfg(feature = "azure_storage_queues")]
mod msal;

use std::fs::File;
use std::io::Read;
use std::sync::Arc;
use tokio::task::JoinSet;
use publishers::PublisherType;
use interfaces::{Publisher, QMessage, RawMessage, Error};

async fn thread_loop(
    mut publisher: impl Publisher,
    settings_ptr: Arc<config::Settings>,
) -> Result<(), Error> {
    // initialise the connection
    let loop_name = publisher.repr();
    let _ = publisher.init().await;
    
    loop {
        let message = publisher.next_message().await;
        if let None = message {
            continue
        };
        println!("{}: Found message", &publisher.repr());
        let q_message: QMessage = match serde_json::from_str(&message.as_ref().unwrap().get_content_str()) {
            Ok(value) => value,
            Err(error) => {
                let error_str = error.to_string();
                if error_str.contains("missing field"){
                    println!("{}: Message does not appear to be a valid QMessage - skipping", &loop_name)
                } else {
                    println!(
                        "{}: Unable to construct valid Qmessage from content. Is likely invalid json: {:?}", 
                        &loop_name, error
                    );
                }
                continue
            }
        };
        // get the message type config
        let (flow_name, deployment_name) = q_message.get_flow_deployment();
        let flow_parameters = q_message.get_flow_parameters();
        // trigger prefect deployment
        let trigger_result = prefect::trigger_prefect_deployment(
            &flow_name, &deployment_name, flow_parameters, &settings_ptr
        ).await;
        match trigger_result {
            Ok(flow_run_name) => {
                println!("{}: Successfully triggered {}/{}: {}", &loop_name, &flow_name, &deployment_name, &flow_run_name);
                if let Some(params) = flow_parameters {
                    println!("{}: with parameters {}", &loop_name, params)
                }
                publisher.task_done(message.unwrap()).await;
            },
            Err(error) => println!(
                "{}: Failed to execute prefect deployment trigger. Got {:?}", 
                &loop_name, error
            )
        };
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
        "File does not appear to be a valid config file"
    )
}

#[tokio::main]
async fn main() -> () {
    let args: Vec<String> = std::env::args().collect();
    if args.len() > 2 {
        println!("Must only provide 1 CLI arguments [config path]");
        return ();
    }
    let file_path = args[1].clone();
    drop(args);
    let config_str = load_config_file_str(file_path);
    let mut config: config::ConfigFile = config_from_str(config_str);
    let _ = config.init().await; // get env derived attrs to pass to threads
    
    println!("Event Handler - main | Preparing queue listener service...");

    // create an async thread for each queue
    let mut spawn_set = JoinSet::new();
    let config_iter = config.iter().cloned();
    let settings_ptr = config.get_settings_ptr();
    for pt in config_iter {
        let pub_name = match pt {
            #[cfg(feature = "azure_storage_queues")]
            PublisherType::AzureStorageQueue(queue_config) => {
                let qc_clone = queue_config.clone();
                let settings_c = settings_ptr.clone();
                spawn_set.spawn( async move{
                    thread_loop(qc_clone, settings_c).await.unwrap();
                });
                queue_config.repr()
            },
            PublisherType::StdInput(pub_config) => {
                let pcc = pub_config.clone();
                let settings_c = settings_ptr.clone();
                spawn_set.spawn( async move{
                    thread_loop(pcc, settings_c).await.unwrap();
                });
                pub_config.repr()
            }
        };
        println!("Event Handler - main | Spawning thread to listen to: {}", &pub_name);
    };
    while let Some(res) = spawn_set.join_next().await {
        // let out = res.unwrap();
        res.unwrap();
    };
    ()
    
}


#[cfg(test)]
mod tests {
    
}
