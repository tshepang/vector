use crate::{
    config::{
        log_schema, DataType, GenerateConfig, Resource, SourceConfig, SourceContext,
        SourceDescription,
    },
    event::{Event, Value},
    sources::util::{
        add_query_parameters, decode_body, Encoding, ErrorMessage, HttpSource, HttpSourceAuthConfig,
    },
    tls::TlsConfig,
};
use bytes::Bytes;
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, net::SocketAddr};

use warp::http::{HeaderMap, HeaderValue};

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct SimpleHttpConfig {
    address: SocketAddr,
    #[serde(default)]
    encoding: Encoding,
    #[serde(default)]
    headers: Vec<String>,
    #[serde(default)]
    query_parameters: Vec<String>,
    tls: Option<TlsConfig>,
    auth: Option<HttpSourceAuthConfig>,
    #[serde(default = "crate::serde::default_true")]
    strict_path: bool,
    #[serde(default = "default_path")]
    path: String,
    #[serde(default = "default_path_key")]
    path_key: String,
}

inventory::submit! {
    SourceDescription::new::<SimpleHttpConfig>("http")
}

impl GenerateConfig for SimpleHttpConfig {
    fn generate_config() -> toml::Value {
        toml::Value::try_from(Self {
            address: "0.0.0.0:8080".parse().unwrap(),
            encoding: Default::default(),
            headers: Vec::new(),
            query_parameters: Vec::new(),
            tls: None,
            auth: None,
            path_key: "path".to_string(),
            path: "/".to_string(),
            strict_path: true,
        })
        .unwrap()
    }
}

fn default_path() -> String {
    "/".to_string()
}

fn default_path_key() -> String {
    "path".to_string()
}

#[derive(Clone)]
struct SimpleHttpSource {
    encoding: Encoding,
    headers: Vec<String>,
    query_parameters: Vec<String>,
    path_key: String,
}

impl HttpSource for SimpleHttpSource {
    fn build_events(
        &self,
        body: Bytes,
        header_map: HeaderMap,
        query_parameters: HashMap<String, String>,
        request_path: &str,
    ) -> Result<Vec<Event>, ErrorMessage> {
        decode_body(body, self.encoding)
            .map(|events| add_headers(events, &self.headers, header_map))
            .map(|events| add_query_parameters(events, &self.query_parameters, query_parameters))
            .map(|events| add_path(events, self.path_key.as_str(), request_path))
            .map(|mut events| {
                // Add source type
                let key = log_schema().source_type_key();
                for event in &mut events {
                    event.as_mut_log().try_insert(key, Bytes::from("http"));
                }
                events
            })
    }
}

#[async_trait::async_trait]
#[typetag::serde(name = "http")]
impl SourceConfig for SimpleHttpConfig {
    async fn build(&self, cx: SourceContext) -> crate::Result<super::Source> {
        let source = SimpleHttpSource {
            encoding: self.encoding,
            headers: self.headers.clone(),
            query_parameters: self.query_parameters.clone(),
            path_key: self.path_key.clone(),
        };
        source.run(
            self.address,
            &self.path.as_str(),
            self.strict_path,
            &self.tls,
            &self.auth,
            cx,
        )
    }

    fn output_type(&self) -> DataType {
        DataType::Log
    }

    fn source_type(&self) -> &'static str {
        "http"
    }

    fn resources(&self) -> Vec<Resource> {
        vec![Resource::tcp(self.address)]
    }
}

fn add_path(mut events: Vec<Event>, key: &str, path: &str) -> Vec<Event> {
    for event in events.iter_mut() {
        event
            .as_mut_log()
            .insert(key, Value::from(path.to_string()));
    }

    events
}

fn add_headers(
    mut events: Vec<Event>,
    headers_config: &[String],
    headers: HeaderMap,
) -> Vec<Event> {
    for header_name in headers_config {
        let value = headers.get(header_name).map(HeaderValue::as_bytes);

        for event in events.iter_mut() {
            event.as_mut_log().insert(
                header_name as &str,
                Value::from(value.map(Bytes::copy_from_slice)),
            );
        }
    }

    events
}

#[cfg(test)]
mod tests {
    use super::{Encoding, SimpleHttpConfig};
    use crate::{
        config::{log_schema, SourceConfig, SourceContext},
        event::{Event, EventStatus, Value},
        test_util::{next_addr, spawn_collect_n, trace_init, wait_for_tcp},
        Pipeline,
    };
    use flate2::{
        write::{DeflateEncoder, GzEncoder},
        Compression,
    };
    use futures::Stream;
    use http::HeaderMap;
    use pretty_assertions::assert_eq;
    use std::collections::BTreeMap;
    use std::io::Write;
    use std::net::SocketAddr;

    #[test]
    fn generate_config() {
        crate::test_util::test_generate_config::<SimpleHttpConfig>();
    }

    async fn source(
        encoding: Encoding,
        headers: Vec<String>,
        query_parameters: Vec<String>,
        path_key: &str,
        path: &str,
        strict_path: bool,
        status: EventStatus,
        acknowledgements: bool,
    ) -> (impl Stream<Item = Event>, SocketAddr) {
        let (sender, recv) = Pipeline::new_test_finalize(status);
        let address = next_addr();
        let path = path.to_owned();
        let path_key = path_key.to_owned();
        let mut context = SourceContext::new_test(sender);
        context.acknowledgements = acknowledgements;
        tokio::spawn(async move {
            SimpleHttpConfig {
                address,
                encoding,
                headers,
                query_parameters,
                tls: None,
                auth: None,
                strict_path,
                path_key,
                path,
            }
            .build(context)
            .await
            .unwrap()
            .await
            .unwrap();
        });
        wait_for_tcp(address).await;
        (recv, address)
    }

    async fn send(address: SocketAddr, body: &str) -> u16 {
        reqwest::Client::new()
            .post(&format!("http://{}/", address))
            .body(body.to_owned())
            .send()
            .await
            .unwrap()
            .status()
            .as_u16()
    }

    async fn send_with_headers(address: SocketAddr, body: &str, headers: HeaderMap) -> u16 {
        reqwest::Client::new()
            .post(&format!("http://{}/", address))
            .headers(headers)
            .body(body.to_owned())
            .send()
            .await
            .unwrap()
            .status()
            .as_u16()
    }

    async fn send_with_query(address: SocketAddr, body: &str, query: &str) -> u16 {
        reqwest::Client::new()
            .post(&format!("http://{}?{}", address, query))
            .body(body.to_owned())
            .send()
            .await
            .unwrap()
            .status()
            .as_u16()
    }

    async fn send_with_path(address: SocketAddr, body: &str, path: &str) -> u16 {
        reqwest::Client::new()
            .post(&format!("http://{}{}", address, path))
            .body(body.to_owned())
            .send()
            .await
            .unwrap()
            .status()
            .as_u16()
    }

    async fn send_bytes(address: SocketAddr, body: Vec<u8>, headers: HeaderMap) -> u16 {
        reqwest::Client::new()
            .post(&format!("http://{}/", address))
            .headers(headers)
            .body(body)
            .send()
            .await
            .unwrap()
            .status()
            .as_u16()
    }

    async fn spawn_ok_collect_n(
        send: impl std::future::Future<Output = u16> + Send + 'static,
        rx: impl Stream<Item = Event> + Unpin,
        n: usize,
    ) -> Vec<Event> {
        spawn_collect_n(async move { assert_eq!(200, send.await) }, rx, n).await
    }

    #[tokio::test]
    async fn http_multiline_text() {
        trace_init();

        let body = "test body\n\ntest body 2";

        let (rx, addr) = source(
            Encoding::default(),
            vec![],
            vec![],
            "http_path",
            "/",
            true,
            EventStatus::Delivered,
            true,
        )
        .await;

        let mut events = spawn_ok_collect_n(send(addr, body), rx, 2).await;

        {
            let event = events.remove(0);
            let log = event.as_log();
            assert_eq!(log[log_schema().message_key()], "test body".into());
            assert!(log.get(log_schema().timestamp_key()).is_some());
            assert_eq!(log[log_schema().source_type_key()], "http".into());
            assert_eq!(log["http_path"], "/".into());
        }
        {
            let event = events.remove(0);
            let log = event.as_log();
            assert_eq!(log[log_schema().message_key()], "test body 2".into());
            assert!(log.get(log_schema().timestamp_key()).is_some());
            assert_eq!(log[log_schema().source_type_key()], "http".into());
            assert_eq!(log["http_path"], "/".into());
        }
    }

    #[tokio::test]
    async fn http_multiline_text2() {
        trace_init();

        //same as above test but with a newline at the end
        let body = "test body\n\ntest body 2\n";

        let (rx, addr) = source(
            Encoding::default(),
            vec![],
            vec![],
            "http_path",
            "/",
            true,
            EventStatus::Delivered,
            true,
        )
        .await;

        let mut events = spawn_ok_collect_n(send(addr, body), rx, 2).await;

        {
            let event = events.remove(0);
            let log = event.as_log();
            assert_eq!(log[log_schema().message_key()], "test body".into());
            assert!(log.get(log_schema().timestamp_key()).is_some());
            assert_eq!(log[log_schema().source_type_key()], "http".into());
            assert_eq!(log["http_path"], "/".into());
        }
        {
            let event = events.remove(0);
            let log = event.as_log();
            assert_eq!(log[log_schema().message_key()], "test body 2".into());
            assert!(log.get(log_schema().timestamp_key()).is_some());
            assert_eq!(log[log_schema().source_type_key()], "http".into());
            assert_eq!(log["http_path"], "/".into());
        }
    }

    #[tokio::test]
    async fn http_multiline_text3() {
        trace_init();

        //same as above test but with a binary encoding
        let body = "test body\n\ntest body 2\n";

        let (rx, addr) = source(
            Encoding::Binary,
            vec![],
            vec![],
            "http_path",
            "/",
            true,
            EventStatus::Delivered,
            true,
        )
        .await;

        let mut events = spawn_ok_collect_n(send(addr, body), rx, 1).await;

        {
            let event = events.remove(0);
            let log = event.as_log();
            assert_eq!(
                log[log_schema().message_key()],
                "test body\n\ntest body 2\n".into()
            );
            assert!(log.get(log_schema().timestamp_key()).is_some());
            assert_eq!(log[log_schema().source_type_key()], "http".into());
            assert_eq!(log["http_path"], "/".into());
        }
    }

    #[tokio::test]
    async fn http_json_parsing() {
        trace_init();

        let (rx, addr) = source(
            Encoding::Json,
            vec![],
            vec![],
            "http_path",
            "/",
            true,
            EventStatus::Delivered,
            true,
        )
        .await;

        let mut events = spawn_collect_n(
            async move {
                assert_eq!(400, send(addr, "{").await); //malformed
                assert_eq!(400, send(addr, r#"{"key"}"#).await); //key without value

                assert_eq!(200, send(addr, "{}").await); //can be one object or array of objects
                assert_eq!(200, send(addr, "[{},{},{}]").await);
            },
            rx,
            2,
        )
        .await;

        assert!(events
            .remove(1)
            .as_log()
            .get(log_schema().timestamp_key())
            .is_some());
        assert!(events
            .remove(0)
            .as_log()
            .get(log_schema().timestamp_key())
            .is_some());
    }

    #[tokio::test]
    async fn http_json_values() {
        trace_init();

        let (rx, addr) = source(
            Encoding::Json,
            vec![],
            vec![],
            "http_path",
            "/",
            true,
            EventStatus::Delivered,
            true,
        )
        .await;

        let mut events = spawn_collect_n(
            async move {
                assert_eq!(200, send(addr, r#"[{"key":"value"}]"#).await);
                assert_eq!(200, send(addr, r#"{"key2":"value2"}"#).await);
            },
            rx,
            2,
        )
        .await;

        {
            let event = events.remove(0);
            let log = event.as_log();
            assert_eq!(log["key"], "value".into());
            assert!(log.get(log_schema().timestamp_key()).is_some());
            assert_eq!(log[log_schema().source_type_key()], "http".into());
            assert_eq!(log["http_path"], "/".into());
        }
        {
            let event = events.remove(0);
            let log = event.as_log();
            assert_eq!(log["key2"], "value2".into());
            assert!(log.get(log_schema().timestamp_key()).is_some());
            assert_eq!(log[log_schema().source_type_key()], "http".into());
            assert_eq!(log["http_path"], "/".into());
        }
    }

    #[tokio::test]
    async fn http_json_dotted_keys() {
        trace_init();

        let (rx, addr) = source(
            Encoding::Json,
            vec![],
            vec![],
            "http_path",
            "/",
            true,
            EventStatus::Delivered,
            true,
        )
        .await;

        let mut events = spawn_collect_n(
            async move {
                assert_eq!(200, send(addr, r#"[{"dotted.key":"value"}]"#).await);
                assert_eq!(
                    200,
                    send(addr, r#"{"nested":{"dotted.key2":"value2"}}"#).await
                );
            },
            rx,
            2,
        )
        .await;

        {
            let event = events.remove(0);
            let log = event.as_log();
            assert_eq!(log.get_flat("dotted.key").unwrap(), &Value::from("value"));
        }
        {
            let event = events.remove(0);
            let log = event.as_log();
            let mut map = BTreeMap::new();
            map.insert("dotted.key2".to_string(), Value::from("value2"));
            assert_eq!(log["nested"], map.into());
        }
    }

    #[tokio::test]
    async fn http_ndjson() {
        trace_init();

        let (rx, addr) = source(
            Encoding::Ndjson,
            vec![],
            vec![],
            "http_path",
            "/",
            true,
            EventStatus::Delivered,
            true,
        )
        .await;

        let mut events = spawn_collect_n(
            async move {
                assert_eq!(400, send(addr, r#"[{"key":"value"}]"#).await); //one object per line

                assert_eq!(
                    200,
                    send(addr, "{\"key1\":\"value1\"}\n\n{\"key2\":\"value2\"}").await
                );
            },
            rx,
            2,
        )
        .await;

        {
            let event = events.remove(0);
            let log = event.as_log();
            assert_eq!(log["key1"], "value1".into());
            assert!(log.get(log_schema().timestamp_key()).is_some());
            assert_eq!(log[log_schema().source_type_key()], "http".into());
            assert_eq!(log["http_path"], "/".into());
        }
        {
            let event = events.remove(0);
            let log = event.as_log();
            assert_eq!(log["key2"], "value2".into());
            assert!(log.get(log_schema().timestamp_key()).is_some());
            assert_eq!(log[log_schema().source_type_key()], "http".into());
            assert_eq!(log["http_path"], "/".into());
        }
    }

    #[tokio::test]
    async fn http_headers() {
        trace_init();

        let mut headers = HeaderMap::new();
        headers.insert("User-Agent", "test_client".parse().unwrap());
        headers.insert("Upgrade-Insecure-Requests", "false".parse().unwrap());

        let (rx, addr) = source(
            Encoding::Ndjson,
            vec![
                "User-Agent".to_string(),
                "Upgrade-Insecure-Requests".to_string(),
                "AbsentHeader".to_string(),
            ],
            vec![],
            "http_path",
            "/",
            true,
            EventStatus::Delivered,
            true,
        )
        .await;

        let mut events = spawn_ok_collect_n(
            send_with_headers(addr, "{\"key1\":\"value1\"}", headers),
            rx,
            1,
        )
        .await;

        {
            let event = events.remove(0);
            let log = event.as_log();
            assert_eq!(log["key1"], "value1".into());
            assert_eq!(log["User-Agent"], "test_client".into());
            assert_eq!(log["Upgrade-Insecure-Requests"], "false".into());
            assert_eq!(log["AbsentHeader"], Value::Null);
            assert_eq!(log["http_path"], "/".into());
            assert!(log.get(log_schema().timestamp_key()).is_some());
            assert_eq!(log[log_schema().source_type_key()], "http".into());
        }
    }

    #[tokio::test]
    async fn http_query() {
        trace_init();
        let (rx, addr) = source(
            Encoding::Ndjson,
            vec![],
            vec![
                "source".to_string(),
                "region".to_string(),
                "absent".to_string(),
            ],
            "http_path",
            "/",
            true,
            EventStatus::Delivered,
            true,
        )
        .await;

        let mut events = spawn_ok_collect_n(
            send_with_query(addr, "{\"key1\":\"value1\"}", "source=staging&region=gb"),
            rx,
            1,
        )
        .await;

        {
            let event = events.remove(0);
            let log = event.as_log();
            assert_eq!(log["key1"], "value1".into());
            assert_eq!(log["source"], "staging".into());
            assert_eq!(log["region"], "gb".into());
            assert_eq!(log["absent"], Value::Null);
            assert_eq!(log["http_path"], "/".into());
            assert!(log.get(log_schema().timestamp_key()).is_some());
            assert_eq!(log[log_schema().source_type_key()], "http".into());
        }
    }

    #[tokio::test]
    async fn http_gzip_deflate() {
        trace_init();

        let body = "test body";

        let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
        encoder.write_all(body.as_bytes()).unwrap();
        let body = encoder.finish().unwrap();

        let mut encoder = DeflateEncoder::new(Vec::new(), Compression::default());
        encoder.write_all(body.as_slice()).unwrap();
        let body = encoder.finish().unwrap();

        let mut headers = HeaderMap::new();
        headers.insert("Content-Encoding", "gzip, deflate".parse().unwrap());

        let (rx, addr) = source(
            Encoding::default(),
            vec![],
            vec![],
            "http_path",
            "/",
            true,
            EventStatus::Delivered,
            true,
        )
        .await;

        let mut events = spawn_ok_collect_n(send_bytes(addr, body, headers), rx, 1).await;

        {
            let event = events.remove(0);
            let log = event.as_log();
            assert_eq!(log[log_schema().message_key()], "test body".into());
            assert!(log.get(log_schema().timestamp_key()).is_some());
            assert_eq!(log[log_schema().source_type_key()], "http".into());
            assert_eq!(log["http_path"], "/".into());
        }
    }

    #[tokio::test]
    async fn http_path() {
        trace_init();
        let (rx, addr) = source(
            Encoding::Ndjson,
            vec![],
            vec![],
            "vector_http_path",
            "/event/path",
            true,
            EventStatus::Delivered,
            true,
        )
        .await;

        let mut events = spawn_ok_collect_n(
            send_with_path(addr, "{\"key1\":\"value1\"}", "/event/path"),
            rx,
            1,
        )
        .await;

        {
            let event = events.remove(0);
            let log = event.as_log();
            assert_eq!(log["key1"], "value1".into());
            assert_eq!(log["vector_http_path"], "/event/path".into());
            assert!(log.get(log_schema().timestamp_key()).is_some());
            assert_eq!(log[log_schema().source_type_key()], "http".into());
        }
    }

    #[tokio::test]
    async fn http_path_no_restriction() {
        trace_init();
        let (rx, addr) = source(
            Encoding::Ndjson,
            vec![],
            vec![],
            "vector_http_path",
            "/event",
            false,
            EventStatus::Delivered,
            true,
        )
        .await;

        let mut events = spawn_collect_n(
            async move {
                assert_eq!(
                    200,
                    send_with_path(addr, "{\"key1\":\"value1\"}", "/event/path1").await
                );
                assert_eq!(
                    200,
                    send_with_path(addr, "{\"key2\":\"value2\"}", "/event/path2").await
                );
            },
            rx,
            2,
        )
        .await;

        {
            let event = events.remove(0);
            let log = event.as_log();
            assert_eq!(log["key1"], "value1".into());
            assert_eq!(log["vector_http_path"], "/event/path1".into());
            assert!(log.get(log_schema().timestamp_key()).is_some());
            assert_eq!(log[log_schema().source_type_key()], "http".into());
        }
        {
            let event = events.remove(0);
            let log = event.as_log();
            assert_eq!(log["key2"], "value2".into());
            assert_eq!(log["vector_http_path"], "/event/path2".into());
            assert!(log.get(log_schema().timestamp_key()).is_some());
            assert_eq!(log[log_schema().source_type_key()], "http".into());
        }
    }

    #[tokio::test]
    async fn http_wrong_path() {
        trace_init();
        let (_rx, addr) = source(
            Encoding::Ndjson,
            vec![],
            vec![],
            "vector_http_path",
            "/",
            true,
            EventStatus::Delivered,
            true,
        )
        .await;

        assert_eq!(
            404,
            send_with_path(addr, "{\"key1\":\"value1\"}", "/event/path").await
        );
    }

    #[tokio::test]
    async fn http_delivery_failure() {
        trace_init();

        let (rx, addr) = source(
            Encoding::default(),
            vec![],
            vec![],
            "http_path",
            "/",
            true,
            EventStatus::Failed,
            true,
        )
        .await;

        spawn_collect_n(
            async move {
                assert_eq!(400, send(addr, "test body\n").await);
            },
            rx,
            1,
        )
        .await;
    }

    #[tokio::test]
    async fn ignores_disabled_acknowledgements() {
        trace_init();

        let (rx, addr) = source(
            Encoding::default(),
            vec![],
            vec![],
            "http_path",
            "/",
            true,
            EventStatus::Failed,
            false,
        )
        .await;

        let events = spawn_collect_n(
            async move {
                assert_eq!(200, send(addr, "test body\n").await);
            },
            rx,
            1,
        )
        .await;

        assert_eq!(events.len(), 1);
    }
}
