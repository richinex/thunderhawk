pub mod appstate;
pub mod config;
pub mod utils;
pub mod factory;
pub mod loadtest;
pub mod tasks;
pub mod cli;

use actix_web::{web, App, HttpResponse, HttpServer, Responder};
use cli::process_http_default_headers;
use config::{load_workflow, Settings, Workflow};
use factory::start_monitoring;
use serde::Deserialize;
use std::{collections::HashMap, sync::Arc};
use tokio::sync::Mutex;
use crate::appstate::AppState;
use crate::cli::build_cli;



// The entry point of the Actix web server.
// Entry point for the Actix web server.
#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Parse command line arguments using clap.
    let matches = build_cli().get_matches();

    // Extract configuration file or directory from CLI arguments.
    let config_file = matches.get_one::<String>("config").map(|s| s.to_string());
    let config_dir = matches.get_one::<String>("config-dir").map(|s| s.to_string());

    // Load workflows based on provided configuration.
    let workflows = load_workflow(config_file, config_dir).await.expect("Failed to load workflows");

    // Extract optional HTTP proxy URL from CLI arguments.
    let http_proxy_url = matches.get_one::<String>("http_proxy_url").map(|s| s.to_string());

    // Process and validate HTTP default headers specified in CLI arguments.
    let http_default_headers = process_http_default_headers(&matches)
        .unwrap_or_else(|err| {
            eprintln!("Error processing HTTP default headers: {}", err);
            std::process::exit(1);
        });

    // Initialize application settings based on CLI arguments.
    let global_settings = Settings {
        monitoring_interval_seconds: matches.get_one::<String>("monitoring_interval_seconds")
            .and_then(|s| s.parse().ok())
            .unwrap_or(60), // Default to 60 seconds if not specified
        log_level: matches.get_one::<String>("log_level")
            .unwrap_or(&"info".to_string()).clone(), // Default to "info" if not specified
        http_timeout_seconds: matches.get_one::<String>("http_timeout_seconds")
            .and_then(|s| s.parse().ok())
            .unwrap_or(20), // Default to 20 seconds if not specified
        http_proxy_url,
        http_default_headers,
    };

    // Initialize logging based on the specified log level.
    global_settings.init_logging();

    // Wrap workflows and settings in Arcs for thread-safe shared access across async tasks.
    let workflows_arc = Arc::new(workflows.into_iter().map(Arc::new).collect::<Vec<_>>());
    let settings_arc = Arc::new(global_settings);

    // Prepare the shared application state for concurrent access.
    let app_state_arc = Arc::new(Mutex::new(AppState {
        monitoring_started: false, // Monitoring has not started initially
        load_test_monitoring_data: Arc::new(Mutex::new(HashMap::new())),
        task_monitoring_data: Arc::new(Mutex::new(HashMap::new())),
    }));


    // Make shared state accessible in Actix web handlers through web::Data.
    let app_state_for_actix = web::Data::new(app_state_arc.clone());
    let workflows_for_actix = web::Data::new(workflows_arc.clone());
    let settings_for_actix = web::Data::new(settings_arc.clone());


    // Set up and run the Actix web server with configured routes and handlers.
    HttpServer::new(move || {
        App::new()
            .app_data(app_state_for_actix.clone())
            .app_data(settings_for_actix.clone())
            .app_data(workflows_for_actix.clone())
            .route("/load_test_results", web::get().to(get_load_test_data))
            .route("/trigger_workflow", web::get().to(trigger_monitoring))
            .route("/task_results", web::get().to(get_task_data))
            .route("/trigger_workflow", web::post().to(trigger_monitoring_via_webhook))
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}

//Separation of Concerns: This approach cleanly separates the concerns of reading data (which might be needed for generating a response)
// from modifying the shared state. It ensures that the operation which modifies the state
//(like marking monitoring_started as false) does not inadvertently depend on or interfere with the data retrieval logic.

// Retrieves and responds with HTTP status data from the shared application state.
async fn get_task_data(data: web::Data<Arc<Mutex<AppState>>>) -> impl actix_web::Responder {
    // Scope for the immutable borrow
    let task_data = {
        let app_state = data.lock().await;
        let task_data_lock = app_state.task_monitoring_data.lock().await;
        // Clone the data to release the lock before modifying AppState
        task_data_lock.clone()
    };

    // Now, modify the AppState outside the scope of the immutable borrow
    let mut app_state = data.lock().await;
    app_state.monitoring_started = false;

    HttpResponse::Ok().json(&task_data)
}



// Handles web requests to retrieve load test data, utilizing shared application state.
async fn get_load_test_data(data: web::Data<Arc<Mutex<AppState>>>) -> impl Responder {
    // Scope for the immutable borrow
    let load_test_data = {
        let app_state = data.lock().await;
        let load_test_data_lock = app_state.load_test_monitoring_data.lock().await;
        // Clone the data to release the lock before modifying AppState
        load_test_data_lock.clone()
    };

    // Now, modify the AppState outside the scope of the immutable borrow
    let mut app_state = data.lock().await;
    app_state.monitoring_started = false;

    HttpResponse::Ok().json(&load_test_data)
}

#[derive(Debug, Deserialize)]
pub struct WebhookPayload {
    workflow_names: Vec<String>, // List of workflow names to trigger
}


async fn trigger_monitoring_via_webhook(
    settings: web::Data<Arc<Settings>>,
    app_state: web::Data<Arc<Mutex<AppState>>>,
    workflows: web::Data<Arc<Vec<Arc<Workflow>>>>,
    payload: web::Json<WebhookPayload>, // Receive the payload as JSON
) -> impl Responder {
    let mut state = app_state.get_ref().lock().await;

    if state.monitoring_started {
        return HttpResponse::Ok().body("Monitoring is already running.");
    }

    // Directly use the filtered Vec<Arc<Workflow>> without wrapping it in an Arc.
    let filtered_workflows = workflows
        .get_ref()
        .iter()
        .filter(|w| payload.workflow_names.contains(&w.name)) // Filter workflows by name
        .cloned()
        .collect::<Vec<_>>();

    if filtered_workflows.is_empty() {
        return HttpResponse::BadRequest().body("No matching workflows found.");
    }

    let settings_clone = Arc::clone(settings.get_ref());
    let app_state_clone = Arc::clone(app_state.get_ref());

    tokio::spawn(async move {
        // Pass filtered_workflows directly to start_monitoring.
        start_monitoring(settings_clone, filtered_workflows, app_state_clone).await;
    });

    state.monitoring_started = true;

    HttpResponse::Ok().body("Monitoring triggered for specified workflows.")
}

// Asynchronously triggers monitoring based on the provided settings, app state, and workflows.
async fn trigger_monitoring(
    settings: web::Data<Arc<Settings>>,
    app_state: web::Data<Arc<Mutex<AppState>>>,
    workflows: web::Data<Arc<Vec<Arc<Workflow>>>>
) -> impl actix_web::Responder {
    let mut state = app_state.get_ref().lock().await;

    // Check if monitoring has already been started
    if state.monitoring_started {
        return HttpResponse::Ok().body("Monitoring is already running.");
    }

    // If monitoring hasn't started, proceed to start it
    let settings_clone = Arc::clone(settings.get_ref());
    let app_state_clone = Arc::clone(app_state.get_ref());
    let workflows_clone = Arc::clone(workflows.get_ref());

    tokio::spawn(async move {
        start_monitoring(settings_clone, (*workflows_clone).clone(), app_state_clone).await;
    });

    // Set the flag to true indicating monitoring has started
    state.monitoring_started = true;

    HttpResponse::Ok().body("Monitoring started.")
}
