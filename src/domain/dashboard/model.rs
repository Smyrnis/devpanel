#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DashboardService {
    Apache,
    MySql,
    Php,
    Composer,
    Node,
}

#[derive(Debug, Clone)]
pub struct DashboardSnapshot {
    pub apache: bool,
    pub mysql: bool,
    pub php: Option<String>,
    pub php_versions: Vec<String>,
    pub apache_uptime: Option<String>,
    pub mysql_uptime: Option<String>,
    pub recent_failures: Vec<String>,
}
