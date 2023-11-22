mod handlers;

#[cfg(test)]
mod tests {
    use crate::handlers::model_router::check_api_status;
    use tokio;

    #[tokio::test]
    async fn test_check_api_status() {
        let openai_status = check_api_status("openai".to_string()).await.unwrap();
        assert_eq!(openai_status, "OK");

        let anthropic_status = check_api_status("anthropic".to_string()).await.unwrap();
        assert_eq!(anthropic_status, "Anthropic API is Operational");

        let unknown_status = check_api_status("unknown".to_string()).await;
        assert!(unknown_status.is_err());
    }
}