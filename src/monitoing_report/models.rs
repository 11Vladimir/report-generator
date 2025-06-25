use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonitoringReportData {
    pub account_inn: String,
    pub account_name: String,
    pub industry: String,
    pub business_segment_label: String,
    pub signal_name: String,
    pub description: Option<String>,
    pub signal_name: String,
    pub created_by: Option<String>,
    pub date_status: String,
    pub monitoring_results_history: Option<Vec<MonitoringHistoryItem>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonitoringHistoryItem {
    pub date_status: String,
    pub status_signal_label: String,
    pub description: Option<String>,
}
