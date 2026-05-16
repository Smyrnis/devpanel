pub async fn run_service_cmd(
    service: String,
    action: String,
    password: String,
) -> crate::messages::Message {
    let result: Result<String, String> =
        crate::sudo_s::common_sudo::systemctl(&password, &action, &service).await;
    crate::messages::Message::Dashboard(crate::messages::DashboardMessage::ServiceResult {
        service,
        action,
        success: result.is_ok(),
        output: result.err().unwrap_or_default(),
    })
}
