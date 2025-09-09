use anyhow::{Context, Result, bail};
use reqwest::{
    blocking::Client,
    header::{self, HeaderName, HeaderValue},
};
use serde::{Deserialize, Serialize};

use crate::git::{self, RevList};

const GITHUB_API: &str = "https://api.github.com";

#[derive(Deserialize)]
struct Repository {
    default_branch: String,
}

#[derive(Serialize)]
struct PullRequest {
    head: String,
    base: String,
    title: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    body: Option<String>,
    maintainer_can_modify: bool,
}

#[derive(Deserialize)]
pub struct PullRequestResponse {
    pub html_url: String,
    pub number: i64,
}

#[derive(Serialize)]
struct Labels {
    labels: Vec<String>,
}

#[derive(Debug, Deserialize)]
struct CommitComparison {
    pub commits: Vec<Commit>,
}

#[derive(Debug, Deserialize)]
struct Commit {
    pub sha: String,
    pub commit: CommitDetails,
}

#[derive(Debug, Deserialize)]
struct CommitDetails {
    pub message: String,
}

pub struct GitHubRepoApiBuilder {
    repository: String,
    token: Option<String>,
}

impl GitHubRepoApiBuilder {
    pub fn new(repository: &str) -> Self {
        Self {
            repository: repository.into(),
            token: None,
        }
    }

    pub fn token(mut self, token: &str) -> Self {
        self.token = Some(token.into());
        self
    }

    pub fn build(self) -> Result<GitHubRepoApi> {
        let mut headers = header::HeaderMap::new();
        if let Some(token) = self.token {
            headers.insert(
                header::AUTHORIZATION,
                format!("Bearer {token}")
                    .parse()
                    .context("Failed to parse token as header value")?,
            );
        }
        headers.insert(
            header::ACCEPT,
            HeaderValue::from_static("application/vnd.github+json"),
        );
        headers.insert(
            HeaderName::from_static("x-github-api-version"),
            HeaderValue::from_static("2022-11-28"),
        );

        let client = Client::builder()
            .user_agent("LonBot")
            .default_headers(headers)
            .build()
            .context("Failed to build the HTTP client")?;

        Ok(GitHubRepoApi {
            client,
            repo_api_url: Self::repo_api_url(&self.repository),
        })
    }

    fn repo_api_url(repo: &str) -> String {
        format!("{GITHUB_API}/repos/{repo}")
    }
}

pub struct GitHubRepoApi {
    client: Client,
    /// The URL to the GitHub API of the specific repo
    repo_api_url: String,
}

impl GitHubRepoApi {
    pub fn builder(repository: &str) -> GitHubRepoApiBuilder {
        GitHubRepoApiBuilder::new(repository)
    }

    pub fn add_labels_to_issue(&self, number: i64, labels: &[String]) -> Result<()> {
        let url = format!("{}/issues/{number}/labels", self.repo_api_url);

        let labels = Labels {
            labels: labels.to_vec(),
        };

        let res = self
            .client
            .post(&url)
            .json(&labels)
            .send()
            .with_context(|| format!("Failed to send GET request to {url}"))?;

        let status = res.status();
        if !status.is_success() {
            bail!("Failed to add labels to {url}: {status}:\n{}", res.text()?)
        }

        Ok(())
    }

    pub fn compare_commits(
        &self,
        old_revision: &str,
        new_revision: &str,
        num_commits: usize,
    ) -> Result<Option<RevList>> {
        let url = format!(
            "{}/compare/{old_revision}...{new_revision}",
            self.repo_api_url
        );

        let res = self
            .client
            .get(&url)
            .send()
            .with_context(|| format!("Failed to send POST request to {url}"))?;

        let status = res.status();
        if !status.is_success() {
            bail!(
                "Failed to get repository information from {url}: {status}:\n{}",
                res.text()?
            )
        }

        let comparison = res.json::<CommitComparison>()?;

        let commits = comparison
            .commits
            .iter()
            .take(num_commits)
            .map(|c| git::Commit::from_str(&c.sha, &c.commit.message));

        Ok(Some(RevList::from_commits(commits)))
    }

    pub fn open_pull_request(
        &self,
        branch: &str,
        title: &str,
        body: Option<String>,
    ) -> Result<PullRequestResponse> {
        let repository = self.get_repository()?;

        let pull_request = PullRequest {
            head: branch.into(),
            base: repository.default_branch.clone(),
            title: title.into(),
            body,
            maintainer_can_modify: true,
        };

        let url = format!("{}/pulls", self.repo_api_url);

        let res = self
            .client
            .post(&url)
            .json(&pull_request)
            .send()
            .with_context(|| format!("Failed to send POST request to {url}"))?;

        let status = res.status();
        if !status.is_success() {
            bail!(
                "Failed to open Pull Request at {url}: {status}:\n{}",
                res.text()?
            )
        }

        let pull_request_response = res.json::<PullRequestResponse>()?;

        Ok(pull_request_response)
    }

    fn get_repository(&self) -> Result<Repository> {
        let url = &self.repo_api_url;

        let res = self
            .client
            .get(url)
            .send()
            .with_context(|| format!("Failed to send GET request to {url}"))?;

        let status = res.status();
        if !status.is_success() {
            bail!(
                "Failed to get repository information from {url}: {status}:\n{}",
                res.text()?
            )
        }

        let repository = res.json::<Repository>()?;

        Ok(repository)
    }
}
