use anyhow::{Context, Result, anyhow};
use winauth::http::Authenticator;

pub enum Method {
    Get,
}

pub struct Response {
    pub status: u16,
    pub body: String,
}

pub fn perform_ntlm_request(method: Method, url: String, headers: &[(String, String)]) -> Result<Response> {
    let client = reqwest::blocking::Client::new();

    let mut out_resp: Option<winauth::http::Response> = None;

    let mut sspi = winauth::windows::NtlmSspiBuilder::new()
        .outbound()
        .build()?;

    let res = loop {        
        let mut builder = client.request(
            match method {
                Method::Get => reqwest::Method::GET,
            },
            &url,
        );

        for (k, v) in headers {
            builder = builder.header(k, v);
        }

        if let Some(out_resp) = out_resp {
            for (k, v) in out_resp.headers {
                builder = builder.header(k, v);
            }
        }

        let res = builder.send().context("Failed to send request")?;

        let ret = sspi
            .http_outgoing_auth(|header| {
                Ok(res
                    .headers()
                    .get_all(header)
                    .into_iter()
                    .map(|x| x.to_str().unwrap())
                    .collect())
            }).map_err(|e| anyhow!(e.to_string()))?;

        match ret {
            winauth::http::AuthState::Response(resp) => {
                out_resp = Some(resp);
            }
            winauth::http::AuthState::Success | winauth::http::AuthState::NotRequested => {
                break res
            }
        }
    };

    Ok(
        Response {
            status: res.status().as_u16(),
            body: res.text()?
        }
    )
}

#[flutter_rust_bridge::frb(init)]
pub fn init_app() {
    flutter_rust_bridge::setup_default_user_utils();
}