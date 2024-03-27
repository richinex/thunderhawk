
# ThunderHawk

ThunderHawk is a command-line tool designed for load testing APIs. It utilizes an Actix Web server to provide HTTP endpoints for triggering workflow processes, fetching task results, and managing load test data. This tool is highly configurable, supporting both file-based and command-line configurations for flexible deployment and testing.

## Features

- **Workflow Management**: Trigger and manage predefined workflows with ease.
- **Flexible Configuration**: Configure through YAML files or command-line arguments.
- **Task Results Retrieval**: Access the results of executed tasks via HTTP endpoints.
- **Load Test Integration**: Initiate and manage load tests as part of the workflow execution.
- **Detailed Logging**: Monitor the application's behavior and performance with configurable logging levels.

## Installation

Ensure you have Rust and Cargo installed. Clone the repository and build the project with `cargo build --release` for optimized performance.

## Modules

- `appstate`: Manages shared application state.
- `config`: Configuration loading and management.
- `utils`: Utility functions and helpers.
- `factory`: Workflow and task factory for processing and monitoring.
- `loadtest`: Load testing components.
- `tasks`: Task definitions and execution logic.
- `cli`: Command-line interface for server configuration and management.

## Features

- **HTTP Endpoint for Workflow Monitoring**: Trigger monitoring of specified workflows with HTTP GET or POST requests.
- **Task Results Retrieval**: Fetch results of completed tasks via a dedicated HTTP endpoint.
- **Load Test Data Management**: Retrieve load test results for analysis and review.
- **Flexible Configuration**: Specify server settings, including monitoring intervals and log levels, via command-line arguments or configuration files.
- **Concurrency and Asynchrony**: Utilizes Rust's async/await features and Actix Web's powerful asynchronous processing capabilities to handle multiple tasks concurrently.


## Usage

Launch the server with `cargo run`, specifying configuration options as needed. Below are some of the available command-line options:

- `--config <FILE>`: Sets a custom configuration file.
- `--config-dir <DIRECTORY>`: Sets the directory from which to load configuration files.
- `--monitoring-interval-seconds <SECONDS>`: Sets the monitoring interval.
- `--log-level <LEVEL>`: Sets the logging level (e.g., info, debug).
- `--http-timeout-seconds <SECONDS>`: Sets the HTTP timeout.
- `--http-proxy-url <URL>`: Sets the HTTP proxy URL.
- `--http-default-header <KEY:VALUE>`: Sets a default HTTP header. Can be used multiple times for multiple headers.

## Configuration Example (`workflow_config.yaml`)

```yaml
name: "User Onboarding Workflow"
apis:
  - name: "Setup Department"
    url: https://jsonplaceholder.typicode.com/todos/1
    method: GET
    expected_field: id
    response_time_threshold: 2000
    load_test: false
```

## Running the Server

Example command to run the server with a specific config file and log level:

```shell
cargo run -- --config ./path/to/workflow_config.yaml --log-level debug
```

## Extending the Workflow Runner

Add new workflow definitions in the configuration file or implement additional functionality within the server to suit your specific needs.

## Contributions

Contributions are welcome! Feel free to submit pull requests or open issues.

## License

This project is open-source and available under the MIT License.
