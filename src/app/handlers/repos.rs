use iced::Task;

use crate::app::App;

use crate::core::error::result_status;

use crate::core::system::{open_terminal_at, xdg_open};

use crate::messages::{Message, ReposMessage};

use crate::tabs::repos::SshStatus;

impl App {
    pub(crate) fn handle_repos(&mut self, msg: ReposMessage) -> Task<Message> {
        match msg {
            ReposMessage::CheckSsh => Task::perform(crate::tabs::repos::check_ssh(), |r| {
                Message::Repos(ReposMessage::SshChecked(
                    r.github_ok,
                    r.github_msg,
                    r.bb_ok,
                    r.bb_msg,
                ))
            }),

            ReposMessage::SshChecked(gok, gmsg, bok, bmsg) => {
                self.repos.github_status = if gok {
                    SshStatus::Connected
                } else {
                    SshStatus::Failed(gmsg)
                };
                self.repos.bitbucket_status = if bok {
                    SshStatus::Connected
                } else {
                    SshStatus::Failed(bmsg)
                };
                Task::none()
            }

            ReposMessage::Fetch => {
                self.repos.fetching = true;
                let root = self.repos.repos_root.clone();
                Task::perform(crate::tabs::repos::fetch_remote_repos(root), |repos| {
                    Message::Repos(ReposMessage::FetchDone(repos))
                })
            }

            ReposMessage::FetchDone(repos) => {
                self.repos.set_repos(repos);
                Task::none()
            }
            ReposMessage::NextPage => {
                let pages = self.repos.remote_repos.len().saturating_sub(1) / self.repos.page_size;
                self.repos.page = (self.repos.page + 1).min(pages);
                Task::none()
            }
            ReposMessage::PrevPage => {
                self.repos.page = self.repos.page.saturating_sub(1);
                Task::none()
            }

            ReposMessage::Clone { ssh_url, name } => {
                self.repos.mark_cloning(&ssh_url, true);
                let root = self.repos.repos_root.clone();
                let result_url = ssh_url.clone();
                Task::perform(
                    crate::tabs::repos::clone_repo(ssh_url, name, root),
                    move |result| {
                        let (ok, msg) = result_status(result);
                        Message::Repos(ReposMessage::CloneDone(ok, msg, result_url.clone()))
                    },
                )
            }

            ReposMessage::CloneDone(ok, msg, ssh_url) => {
                if ok {
                    self.repos.mark_cloned(&ssh_url);
                } else {
                    self.repos.mark_cloning(&ssh_url, false);
                }
                self.repos.status_msg = Some((ok, msg.clone()));
                self.show_toast(msg, ok)
            }

            ReposMessage::OpenCloned(name) => {
                open_terminal_at(&format!("{}/{}", self.repos.repos_root, name));
                Task::none()
            }
            ReposMessage::OpenEditor(name) => {
                let path = format!("{}/{}", self.repos.repos_root, name);
                let cmd = self.config_tab.settings.editor_command.clone();
                match crate::core::system::open_in_editor(&cmd, &path) {
                    Ok(_) => Task::none(),
                    Err(e) => self.show_toast(e, false),
                }
            }

            ReposMessage::SearchChanged(v) => {
                self.repos.search_query = v;
                Task::none()
            }
            ReposMessage::SetFilter(f) => {
                self.repos.active_filter = f;
                Task::none()
            }
            ReposMessage::OpenRoot => {
                let _ = xdg_open(&self.repos.repos_root);
                Task::none()
            }
        }
    }
}
