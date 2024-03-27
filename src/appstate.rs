use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;
use crate::loadtest::LoadTestMonitoringData;
use crate::tasks::MonitoringData;

#[derive(Debug)]
pub struct AppState {
    /// Indicates whether the monitoring task has been started.
    pub monitoring_started: bool,
    /// Monitoring data for load tests, organized by workflow name and then by API URL.
    pub load_test_monitoring_data: Arc<Mutex<HashMap<String, HashMap<String, LoadTestMonitoringData>>>>,
    /// Monitoring data for tasks, organized by workflow name and then by API URL.
    pub task_monitoring_data: Arc<Mutex<HashMap<String, HashMap<String, MonitoringData>>>>,
}
