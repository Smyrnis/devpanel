// src/tabs/repos/mod.rs — state, data types, public API

pub mod backend;
pub mod view;

// Re-export backend functions so callers can use `tabs::repos::check_ssh()` etc.
pub use backend::{check_ssh, clone_repo, fetch_remote_repos};

use iced::Element;
use crate::Message;

// ── Data types ────────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq)]
pub enum Provider {
    GitHub,
    Bitbucket,
}

impl Provider {
    pub fn label(&self) -> &'static str {
        match self {
            Provider::GitHub    => "GitHub",
            Provider::Bitbucket => "Bitbucket",
        }
    }
}

#[derive(Debug, Clone)]
pub struct RemoteRepo {
    pub name:       String,   // e.g. "my-project"
    pub full_name:  String,   // e.g. "username/my-project"
    pub ssh_url:    String,   // e.g. "git@github.com:username/my-project.git"
    pub provider:   Provider,
    pub is_cloned:  bool,
    pub is_cloning: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub enum SshStatus {
    Unknown,
    Connected,
    Failed(String),
}

#[derive(Debug, Clone, PartialEq)]
pub enum ProviderFilter {
    All,
    GitHub,
    Bitbucket,
}

// ── Tab state ─────────────────────────────────────────────────────────────

pub struct ReposTab {
    pub repos_root:       String,
    pub remote_repos:     Vec<RemoteRepo>,
    pub fetching:         bool,
    pub github_status:    SshStatus,
    pub bitbucket_status: SshStatus,
    pub search_query:     String,
    pub status_msg:       Option<(bool, String)>,
    pub active_filter:    ProviderFilter,
}

impl ReposTab {
    pub fn new(repos_root: String, _unused: String) -> Self {
        Self {
            repos_root,
            remote_repos:     Vec::new(),
            fetching:         false,
            github_status:    SshStatus::Unknown,
            bitbucket_status: SshStatus::Unknown,
            search_query:     String::new(),
            status_msg:       None,
            active_filter:    ProviderFilter::All,
        }
    }

    pub fn set_repos(&mut self, repos: Vec<RemoteRepo>) {
        self.fetching    = false;
        self.remote_repos = repos;
    }

    pub fn mark_cloning(&mut self, ssh_url: &str, cloning: bool) {
        if let Some(r) = self.remote_repos.iter_mut().find(|r| r.ssh_url == ssh_url) {
            r.is_cloning = cloning;
        }
    }

    pub fn mark_cloned(&mut self, ssh_url: &str) {
        if let Some(r) = self.remote_repos.iter_mut().find(|r| r.ssh_url == ssh_url) {
            r.is_cloning = false;
            r.is_cloned  = true;
        }
    }

    pub fn view(&self) -> Element<'_, Message> {
        view::render(self)
    }
}
