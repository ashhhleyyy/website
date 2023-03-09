use maud::PreEscaped;
use reqwest::{Client, ClientBuilder};
use serde::{Deserialize, Serialize};

use crate::{apis::USER_AGENT, error::Result};

#[derive(Clone)]
pub struct MediawikiClient<S: ClientState> {
    client: Client,
    domain: String,
    state: S,
}

impl MediawikiClient<Credentials> {
    pub fn new(domain: String, username: String, password: String) -> Self {
        let client = ClientBuilder::new()
            .user_agent(USER_AGENT)
            .cookie_store(true)
            .build()
            .expect("failed to build client");
        Self {
            state: Credentials { username, password },
            client,
            domain,
        }
    }

    async fn get_login_token(&self) -> Result<String> {
        let url = format!(
            "https://{domain}/w/api.php?action=query&meta=tokens&format=json&type=login",
            domain = self.domain
        );
        let res: QueryResponse = self
            .client
            .get(url)
            .send()
            .await?
            .error_for_status()?
            .json()
            .await?;

        Ok(res.query.tokens.login_token)
    }

    pub async fn log_in(self, allowed_pages: Vec<String>) -> Result<MediawikiClient<LoggedIn>> {
        let login_token = self.get_login_token().await?;
        let url = format!("https://{domain}/w/api.php", domain = self.domain);
        let _res = self
            .client
            .post(url)
            .form(&LoginForm {
                action: "login",
                lgname: self.state.username,
                lgpassword: self.state.password,
                lgtoken: login_token,
                format: "json",
            })
            .send()
            .await?
            .error_for_status()?;

        Ok(MediawikiClient {
            client: self.client,
            domain: self.domain,
            state: LoggedIn { allowed_pages },
        })
    }
}

impl MediawikiClient<LoggedIn> {
    pub async fn get_page(
        &self,
        page_title: String,
    ) -> Result<Option<(String, PreEscaped<String>)>> {
        if !self.state.allowed_pages.contains(&page_title) {
            return Ok(None);
        }

        let url = format!(
            "https://{domain}/w/api.php?action=parse&format=json&page={page_title}",
            domain = self.domain,
        );

        let res: ParseResponse = self
            .client
            .get(url)
            .send()
            .await?
            .error_for_status()?
            .json()
            .await?;

        Ok(Some((res.parse.title, PreEscaped(res.parse.text.content))))
    }
}

pub trait ClientState {}

#[derive(Clone)]
pub struct Credentials {
    username: String,
    password: String,
}

#[derive(Clone)]
pub struct LoggedIn {
    allowed_pages: Vec<String>,
}

impl ClientState for Credentials {}
impl ClientState for LoggedIn {}

#[derive(Deserialize)]
struct QueryResponse {
    query: TokensResponse,
}

#[derive(Deserialize)]
struct TokensResponse {
    tokens: Tokens,
}

#[derive(Deserialize)]
struct Tokens {
    #[serde(rename = "logintoken")]
    login_token: String,
}

#[derive(Serialize)]
struct LoginForm {
    action: &'static str,
    lgname: String,
    lgpassword: String,
    lgtoken: String,
    format: &'static str,
}

#[derive(Deserialize)]
pub struct ParseResponse {
    pub parse: Parse,
}

#[derive(Deserialize)]
pub struct Parse {
    pub title: String,
    pub revid: i64,
    pub text: Text,
}

#[derive(Deserialize)]
pub struct Text {
    #[serde(rename = "*")]
    pub content: String,
}
