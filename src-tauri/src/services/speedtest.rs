use futures::future::join_all;
use reqwest::{Client, Url};
use serde::Serialize;
use std::time::{Duration, Instant};

use crate::error::AppError;

const DEFAULT_TIMEOUT_SECS: u64 = 8;
const MAX_TIMEOUT_SECS: u64 = 30;
const MIN_TIMEOUT_SECS: u64 = 2;

/// 端点测速结果
#[derive(Debug, Clone, Serialize)]
pub struct EndpointLatency {
    pub url: String,
    pub latency: Option<u128>,
    pub status: Option<u16>,
    pub error: Option<String>,
}

/// 网络测速相关业务
pub struct SpeedtestService;

impl SpeedtestService {
    /// 测试一组端点的响应延迟。
    pub async fn test_endpoints(
        urls: Vec<String>,
        timeout_secs: Option<u64>,
    ) -> Result<Vec<EndpointLatency>, AppError> {
        if urls.is_empty() {
            return Ok(vec![]);
        }

        let timeout = Self::sanitize_timeout(timeout_secs);
        let client = Self::build_client(timeout)?;

        let tasks = urls.into_iter().map(|raw_url| {
            let client = client.clone();
            async move {
                let trimmed = raw_url.trim().to_string();
                if trimmed.is_empty() {
                    return EndpointLatency {
                        url: raw_url,
                        latency: None,
                        status: None,
                        error: Some("URL 不能为空".to_string()),
                    };
                }

                let parsed_url = match Url::parse(&trimmed) {
                    Ok(url) => url,
                    Err(err) => {
                        return EndpointLatency {
                            url: trimmed,
                            latency: None,
                            status: None,
                            error: Some(format!("URL 无效: {err}")),
                        };
                    }
                };

                // 先进行一次热身请求，忽略结果，仅用于复用连接/绕过首包惩罚。
                let _ = client.get(parsed_url.clone()).send().await;

                // 第二次请求开始计时，并将其作为结果返回。
                let start = Instant::now();
                match client.get(parsed_url).send().await {
                    Ok(resp) => EndpointLatency {
                        url: trimmed,
                        latency: Some(start.elapsed().as_millis()),
                        status: Some(resp.status().as_u16()),
                        error: None,
                    },
                    Err(err) => {
                        let status = err.status().map(|s| s.as_u16());
                        let error_message = if err.is_timeout() {
                            "请求超时".to_string()
                        } else if err.is_connect() {
                            "连接失败".to_string()
                        } else {
                            err.to_string()
                        };

                        EndpointLatency {
                            url: trimmed,
                            latency: None,
                            status,
                            error: Some(error_message),
                        }
                    }
                }
            }
        });

        Ok(join_all(tasks).await)
    }

    fn build_client(timeout_secs: u64) -> Result<Client, AppError> {
        Client::builder()
            .timeout(Duration::from_secs(timeout_secs))
            .redirect(reqwest::redirect::Policy::limited(5))
            .user_agent("cc-switch-speedtest/1.0")
            .build()
            .map_err(|e| AppError::Message(format!("创建 HTTP 客户端失败: {e}")))
    }

    fn sanitize_timeout(timeout_secs: Option<u64>) -> u64 {
        let secs = timeout_secs.unwrap_or(DEFAULT_TIMEOUT_SECS);
        secs.clamp(MIN_TIMEOUT_SECS, MAX_TIMEOUT_SECS)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sanitize_timeout_clamps_values() {
        assert_eq!(
            SpeedtestService::sanitize_timeout(Some(1)),
            MIN_TIMEOUT_SECS
        );
        assert_eq!(
            SpeedtestService::sanitize_timeout(Some(999)),
            MAX_TIMEOUT_SECS
        );
        assert_eq!(
            SpeedtestService::sanitize_timeout(Some(10)),
            10.clamp(MIN_TIMEOUT_SECS, MAX_TIMEOUT_SECS)
        );
        assert_eq!(
            SpeedtestService::sanitize_timeout(None),
            DEFAULT_TIMEOUT_SECS
        );
    }

    #[test]
    fn test_endpoints_handles_empty_list() {
        let result =
            tauri::async_runtime::block_on(SpeedtestService::test_endpoints(Vec::new(), Some(5)))
                .expect("empty list should succeed");
        assert!(result.is_empty());
    }

    #[test]
    fn test_endpoints_reports_invalid_url() {
        let result = tauri::async_runtime::block_on(SpeedtestService::test_endpoints(
            vec!["not a url".into(), "".into()],
            None,
        ))
        .expect("invalid inputs should still succeed");

        assert_eq!(result.len(), 2);
        assert!(
            result[0]
                .error
                .as_deref()
                .unwrap_or_default()
                .starts_with("URL 无效"),
            "invalid url should yield parse error"
        );
        assert_eq!(
            result[1].error.as_deref(),
            Some("URL 不能为空"),
            "empty url should report validation error"
        );
    }
}
