//! Authorization/Authentication for Domo API.
//!
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json;
use std::borrow::Cow;
use std::collections::HashMap;
use std::env;
use std::io::prelude::*;

/// Domo auth token
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DomoToken {
    pub access_token: String,
    pub token_type: String,
    pub expires_in: u32,
    pub scope: String,
    pub customer: String,
    pub env: String,
    #[serde(rename = "userId")]
    pub user_id: u32,
    pub role: String,
    pub jti: String,
    pub domain: String,
}

/// `OAuth` authorization scopes for the Domo API
#[derive(Default)]
pub struct DomoScope {
    pub data: bool,
    pub user: bool,
    pub audit: bool,
    pub dashboard: bool,
    pub buzz: bool,
    pub account: bool,
    pub workflow: bool,
}

/// Object to use to store/retrieve access tokens for Domo API.
pub struct DomoClientAppCredentials {
    pub client_id: String,
    pub client_secret: String,
    pub token: Option<DomoToken>,
    pub domo_scope: DomoScope,
}

impl DomoToken {
    pub fn default() -> Self {
        Self {
            access_token: String::new(),
            token_type: String::new(),
            expires_in: 0_u32,
            scope: String::new(),
            customer: String::new(),
            env: String::new(),
            jti: String::new(),
            user_id: 0_u32,
            role: String::new(),
            domain: String::new(),
        }
    }

    pub fn access_token(mut self, access_token: &str) -> Self {
        self.access_token = access_token.to_string();
        self
    }

    pub fn token_type(mut self, token_type: &str) -> Self {
        self.token_type = token_type.to_string();
        self
    }

    pub fn expires_in(mut self, expires_in: u32) -> Self {
        self.expires_in = expires_in;
        self
    }

    pub fn scope(mut self, scope: &str) -> Self {
        self.scope = scope.to_string();
        self
    }
}

impl DomoClientAppCredentials {
    pub fn default() -> Self {
        let client_id = env::var("CLIENT_ID")
            // .context("No CLIENT_ID Env Var found")
            .unwrap_or_default();
        let client_secret = env::var("CLIENT_SECRET")
            // .context("No CLIENT_SECRET Env Var found")
            .unwrap_or_default();
        let data_scope: bool = env::var("DATA_SCOPE").is_ok();
        let user_scope: bool = env::var("USER_SCOPE").is_ok();
        let audit_scope: bool = env::var("AUDIT_SCOPE").is_ok();
        let dashboard_scope: bool = env::var("DASHBOARD_SCOPE").is_ok();
        if data_scope && user_scope && audit_scope && dashboard_scope {
            let scope = DomoScope {
                data: data_scope,
                user: user_scope,
                audit: audit_scope,
                dashboard: dashboard_scope,
                buzz: false,
                account: false,
                workflow: false,
            };
            Self {
                client_id,
                client_secret,
                token: None,
                domo_scope: scope,
            }
        } else {
            let scope = DomoScope {
                data: true,
                user: false,
                audit: false,
                dashboard: false,
                buzz: false,
                account: false,
                workflow: false,
            };
            Self {
                client_id,
                client_secret,
                token: None,
                domo_scope: scope,
            }
        }
    }

    pub fn client_id(mut self, client_id: &str) -> Self {
        self.client_id = client_id.to_string();
        self
    }

    pub fn client_secret(mut self, client_secret: &str) -> Self {
        self.client_secret = client_secret.to_string();
        self
    }

    pub fn client_scope(mut self, domo_scope: DomoScope) -> Self {
        self.domo_scope = domo_scope;
        self
    }

    pub fn with_user_scope(mut self) -> Self {
        self.domo_scope.user = true;
        self
    }

    pub fn with_data_scope(mut self) -> Self {
        self.domo_scope.data = true;
        self
    }
    pub fn with_audit_scope(mut self) -> Self {
        self.domo_scope.audit = true;
        self
    }
    pub fn with_dashboard_scope(mut self) -> Self {
        self.domo_scope.dashboard = true;
        self
    }
    pub fn with_buzz_scope(mut self) -> Self {
        self.domo_scope.buzz = true;
        self
    }
    pub fn with_workflow_scope(mut self) -> Self {
        self.domo_scope.workflow = true;
        self
    }
    pub fn with_account_scope(mut self) -> Self {
        self.domo_scope.account = true;
        self
    }

    pub fn token_info(mut self, token: DomoToken) -> Self {
        self.token = Some(token);
        self
    }

    pub fn build(self) -> Self {
        const ERROR_MESSAGE: &str = "Set your Domo API Credentials. You can do this by setting environment variables in `.env` file:
        CLIENT_ID='domo-client-id'
        CLIENT_SECRET='domo-client-secret'";

        let empty_flag = if self.client_id.is_empty() {
            true
        } else {
            self.client_secret.is_empty()
        };

        if empty_flag {
            eprintln!("{}", ERROR_MESSAGE);
        } else {
            //debug!("client_id and client_secret found");
        }
        self
    }

    /// Get cached Domo auth token or authenticate and retrieve a new one
    pub fn get_access_token(&self) -> String {
        match self.token {
            Some(ref token) => token.access_token.to_owned(),
            None => {
                match self.request_access_token() {
                    Some(new_token) => {
                        //debug!("Token: {:?}", &new_token);
                        new_token.access_token
                    }
                    None => String::new(),
                }
            }
        }
    }

    fn request_access_token(&self) -> Option<DomoToken> {
        let mut payload = HashMap::new();
        payload.insert("grant_type", "client_credentials");
        let mut scopes = "".to_string();
        if self.domo_scope.data {
            if !scopes.is_empty() {
                scopes += &"%20".to_string()
            }
            scopes += &"data".to_string();
        }
        if self.domo_scope.user {
            if !scopes.is_empty() {
                scopes += &"%20".to_string()
            }
            scopes += &"user".to_string();
        }
        if self.domo_scope.audit {
            if !scopes.is_empty() {
                scopes += &"%20".to_string()
            }
            scopes += &"audit".to_string();
        }
        if self.domo_scope.dashboard {
            if !scopes.is_empty() {
                scopes += &"%20".to_string()
            }
            scopes += &"dashboard".to_string();
        }

        if let Some(token) = self.fetch_access_token(&self.client_id, &self.client_secret, &scopes)
        {
            Some(token)
        } else {
            None
        }
    }

    fn fetch_access_token(
        &self,
        client_id: &str,
        client_secret: &str,
        params: &str,
    ) -> Option<DomoToken> {
        fetch_access_token(client_id, client_secret, params)
    }
}

fn fetch_access_token(client_id: &str, client_secret: &str, params: &str) -> Option<DomoToken> {
    let client = Client::new();
    let url: Cow<'_, str> = [
        "https://api.domo.com/oauth/token?grant_type=client_credentials&scope=",
        params,
    ]
    .concat()
    .into();
    let mut response = client
        .post(&url.into_owned())
        .basic_auth(client_id, Some(client_secret))
        .send()
        .expect("token request failed");
    let mut buf = String::new();
    response
        .read_to_string(&mut buf)
        .expect("failed to read response");
    if response.status().is_success() {
        let token: DomoToken = serde_json::from_str(&buf).unwrap();
        Some(token)
    } else {
        eprintln!("Error getting Domo Token");
        //error!("fetch access token request failed");
        //error!("{:?}", response);
        None
    }
}
