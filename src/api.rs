#![allow(dead_code)]

use std::time::Duration;

use anyhow::{bail, Result};
use reqwest::Client;
use serde::{Deserialize, Serialize};

pub struct ApiClient {
    base_url: String,
    token: String,
    client: Client,
}

#[derive(Deserialize, Clone, Debug)]
pub struct User {
    #[serde(deserialize_with = "flexible_i64")]
    pub id: i64,
    pub name: String,
    pub email: String,
    #[serde(default)]
    pub color: String,
}

#[derive(Deserialize, Clone, Debug)]
pub struct Project {
    pub id: i64,
    pub name: String,
    #[serde(default)]
    pub description: String,
}

#[derive(Deserialize, Clone, Debug)]
pub struct Task {
    #[serde(deserialize_with = "flexible_i64")]
    pub id: i64,
    pub name: String,
    #[serde(default = "default_status")]
    pub status: String,
    #[serde(deserialize_with = "flexible_i64")]
    pub project_id: i64,
}

fn default_status() -> String {
    "to-do".to_string()
}

#[derive(Deserialize, Clone, Debug)]
pub struct TimeEntry {
    pub id: i64,
    pub project_id: i64,
    pub task_id: i64,
    #[serde(default)]
    pub task_name: String,
    pub user_id: i64,
    pub started_at: String,
    pub stopped_at: Option<String>,
    pub paused_at: Option<String>,
    #[serde(default)]
    pub paused_duration_ms: i64,
}

#[derive(Deserialize)]
struct MeResponse {
    user: User,
}

#[derive(Deserialize)]
struct ProjectsResponse {
    projects: Vec<Project>,
}

#[derive(Deserialize)]
struct TasksResponse {
    tasks: Vec<Task>,
}

#[derive(Deserialize)]
struct RunningResponse {
    entry: Option<TimeEntry>,
}

#[derive(Deserialize)]
struct EntriesResponse {
    entries: Vec<TimeEntry>,
}

#[derive(Deserialize)]
struct ErrorBody {
    error: ErrorDetail,
}

#[derive(Deserialize)]
struct ErrorDetail {
    message: String,
}

#[derive(Serialize)]
struct StartPayload {
    project_id: i64,
    task_id: i64,
}

#[derive(Serialize)]
struct CreateTaskPayload {
    name: String,
}

#[derive(Deserialize)]
struct CreateTaskResponse {
    task: Task,
}

// user_id in responses is a string from the Go backend
fn flexible_i64<'de, D>(deserializer: D) -> std::result::Result<i64, D::Error>
where
    D: serde::Deserializer<'de>,
{
    use serde::de;
    struct Visitor;
    impl<'de> de::Visitor<'de> for Visitor {
        type Value = i64;
        fn expecting(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
            f.write_str("an integer or string-encoded integer")
        }
        fn visit_i64<E: de::Error>(self, v: i64) -> std::result::Result<i64, E> {
            Ok(v)
        }
        fn visit_u64<E: de::Error>(self, v: u64) -> std::result::Result<i64, E> {
            Ok(v as i64)
        }
        fn visit_str<E: de::Error>(self, v: &str) -> std::result::Result<i64, E> {
            v.parse().map_err(de::Error::custom)
        }
    }
    deserializer.deserialize_any(Visitor)
}

impl ApiClient {
    pub fn new(base_url: &str, token: &str) -> Self {
        let client = Client::builder()
            .timeout(Duration::from_secs(15))
            .build()
            .unwrap_or_default();
        Self {
            base_url: base_url.trim_end_matches('/').to_string(),
            token: token.to_string(),
            client,
        }
    }

    fn url(&self, path: &str) -> String {
        format!("{}{}", self.base_url, path)
    }

    async fn check(resp: reqwest::Response) -> Result<reqwest::Response> {
        if resp.status().is_success() {
            return Ok(resp);
        }
        let status = resp.status();
        let body = resp.text().await.unwrap_or_default();
        if let Ok(e) = serde_json::from_str::<ErrorBody>(&body) {
            bail!("{}: {}", status, e.error.message);
        }
        bail!("{}: {}", status, body);
    }

    pub async fn me(&self) -> Result<User> {
        let resp = self
            .client
            .get(self.url("/users/me"))
            .bearer_auth(&self.token)
            .send()
            .await?;
        let data: MeResponse = Self::check(resp).await?.json().await?;
        Ok(data.user)
    }

    pub async fn projects(&self) -> Result<Vec<Project>> {
        let resp = self
            .client
            .get(self.url("/projects"))
            .bearer_auth(&self.token)
            .send()
            .await?;
        let data: ProjectsResponse = Self::check(resp).await?.json().await?;
        Ok(data.projects)
    }

    pub async fn tasks(&self, project_id: i64) -> Result<Vec<Task>> {
        let resp = self
            .client
            .get(self.url(&format!("/projects/{}/tasks", project_id)))
            .bearer_auth(&self.token)
            .send()
            .await?;
        let data: TasksResponse = Self::check(resp).await?.json().await?;
        Ok(data.tasks)
    }

    pub async fn running_entry(&self) -> Result<Option<TimeEntry>> {
        let resp = self
            .client
            .get(self.url("/time-entries/running"))
            .bearer_auth(&self.token)
            .send()
            .await?;
        let data: RunningResponse = Self::check(resp).await?.json().await?;
        Ok(data.entry)
    }

    pub async fn start(&self, project_id: i64, task_id: i64) -> Result<TimeEntry> {
        let resp = self
            .client
            .post(self.url("/time-entries/start"))
            .bearer_auth(&self.token)
            .json(&StartPayload {
                project_id,
                task_id,
            })
            .send()
            .await?;
        Ok(Self::check(resp).await?.json().await?)
    }

    pub async fn stop(&self) -> Result<TimeEntry> {
        let resp = self
            .client
            .post(self.url("/time-entries/stop"))
            .bearer_auth(&self.token)
            .send()
            .await?;
        Ok(Self::check(resp).await?.json().await?)
    }

    pub async fn pause(&self) -> Result<TimeEntry> {
        let resp = self
            .client
            .post(self.url("/time-entries/pause"))
            .bearer_auth(&self.token)
            .send()
            .await?;
        Ok(Self::check(resp).await?.json().await?)
    }

    pub async fn resume(&self) -> Result<TimeEntry> {
        let resp = self
            .client
            .post(self.url("/time-entries/resume"))
            .bearer_auth(&self.token)
            .send()
            .await?;
        Ok(Self::check(resp).await?.json().await?)
    }

    pub async fn create_task(&self, project_id: i64, name: &str) -> Result<Task> {
        let resp = self
            .client
            .post(self.url(&format!("/projects/{}/tasks", project_id)))
            .bearer_auth(&self.token)
            .json(&CreateTaskPayload {
                name: name.to_string(),
            })
            .send()
            .await?;
        let body = Self::check(resp).await?.text().await?;
        if let Ok(data) = serde_json::from_str::<CreateTaskResponse>(&body) {
            return Ok(data.task);
        }
        if let Ok(task) = serde_json::from_str::<Task>(&body) {
            return Ok(task);
        }
        bail!("unexpected create-task response: {}", body);
    }

    pub async fn entries(&self, user_id: Option<i64>) -> Result<Vec<TimeEntry>> {
        let mut url = self.url("/time-entries");
        if let Some(uid) = user_id {
            url.push_str(&format!("?user_id={}", uid));
        }
        let resp = self
            .client
            .get(&url)
            .bearer_auth(&self.token)
            .send()
            .await?;
        let data: EntriesResponse = Self::check(resp).await?.json().await?;
        Ok(data.entries)
    }
}

impl TimeEntry {
    pub fn elapsed_seconds(&self) -> i64 {
        let started = chrono::DateTime::parse_from_rfc3339(&self.started_at).unwrap_or_default();

        let end = if let Some(ref stopped) = self.stopped_at {
            chrono::DateTime::parse_from_rfc3339(stopped).unwrap_or_default()
        } else if let Some(ref paused) = self.paused_at {
            chrono::DateTime::parse_from_rfc3339(paused).unwrap_or_default()
        } else {
            chrono::Utc::now().fixed_offset()
        };

        let total = (end - started).num_seconds();
        let paused = self.paused_duration_ms / 1000;
        (total - paused).max(0)
    }

    pub fn elapsed_display(&self) -> String {
        let secs = self.elapsed_seconds();
        let h = secs / 3600;
        let m = (secs % 3600) / 60;
        let s = secs % 60;
        format!("{:02}:{:02}:{:02}", h, m, s)
    }

    pub fn is_paused(&self) -> bool {
        self.paused_at.is_some() && self.stopped_at.is_none()
    }

    pub fn status_label(&self) -> &str {
        if self.stopped_at.is_some() {
            "Stopped"
        } else if self.paused_at.is_some() {
            "Paused"
        } else {
            "Running"
        }
    }
}
