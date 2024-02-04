# Simple Prefect Event Handler

This is an asynchronous event handler written in Rust that is used to handle events that should kick off a prefect flow. It is a simple handler that listens to some queue/socket-like source to kick off prefect flows.

## Why
As an open source user of Prefect, there is no native support for an event-driven architecture (without having to use prefect cloud). Working in a business with incoming events that should trigger a flow, I needed a way to listen to these events and kick off a flow in Prefect all within our private network.

## How it works
The event handler is a simple tokio rust application that asynchronously listens to multiple queues and can kick off a prefect flow depending on the type of message received to any of the target message queues.

## Design
It was written in rust primarily for speed and reliability and to take advantage of the async/await feature and the tokio runtime. 

The runtime also has very low memory consumption and as this is a long running process that is effectively a middleman between different services, it is important that it is as efficient as possible and does not consume too many resources as it runs on the same VMs as the other services.

## Usage
The application works by using a JSON config file for the queue-message type-flow mapping. A config is passed to each async thread thats created to listen to a queue. 

Here's an example:
`example-config.json`
```json
{
    "threads": [
        {
            "publisher_type": "AzureStorageQueue",
            "storage_account": "storage-account-name",
            "queue_name": "test",
            "message_flow_actions": {
                "SampleMsg": "Flow Name/deployment-name"
            }
        }
    ]
}
```

The binary listener is then run using the following command. Each threads runs forever until a SIGKILL is sent to the main process.
```bash
./prefect-event-handler example-config.json
```

Messages should be received in JSON format: Eg.
```json
{"message_type": "SampleMsg"}
```
 Now try dropping a message into the stdinput for the fllow that takes params
```json
{"message_type": "ParamsMsg", "payload": {"name": "Gordon Bennett"}}
```
## Setup

A few environment variables are required to get started.
The first is the URI to your prefect instance. (If you're running this application on the same machine as your prefect server, this is commonly `http://127.0.0.1:4200/api`)
```bash
export PREFECT_API_URI="https://your-prefect-server@example.com/api"
```

### Server Authentication
> NB. Only applies if you've explicitly added authentication to your prefect server.
if the 

### Publisher Authentication
#### Azure
The application uses the `DefaultAzureCredential` to authenticate with Azure storage accounts. This means that it will use the environment variables or the managed identity of the VM it is running on to authenticate.


## Main Features
- [x] Add Azure Storage Queue support
- [ ] Add a logging system to log the messages received and the flows kicked off
- [ ] Add ØMQ support
- [ ] Add Kafka Support

## Building
To build the event handler, you need to have rust installed. You can install rust using rustup. Once you have rust installed, you can build the event handler using the following command:
```bash
cargo build --release
```
