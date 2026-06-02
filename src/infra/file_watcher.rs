use crate::messages::{Message, VHostsMessage};
use iced::{Subscription, stream};
use notify::{RecommendedWatcher, RecursiveMode, Watcher};
use std::path::{Path, PathBuf};

pub fn vhost_config(path: String) -> Subscription<Message> {
    Subscription::run_with_id(
        ("vhost-config-watch", path.clone()),
        watch_vhost_config(path),
    )
}

fn watch_vhost_config(path: String) -> impl iced::futures::Stream<Item = Message> {
    stream::channel(100, move |mut output| async move {
        let watched_file = PathBuf::from(path);
        let watch_dir = watched_file
            .parent()
            .map(Path::to_path_buf)
            .unwrap_or_else(|| PathBuf::from("."));

        let callback_file = watched_file.clone();
        let watcher_result: notify::Result<RecommendedWatcher> =
            notify::recommended_watcher(move |event: notify::Result<notify::Event>| {
                let Ok(event) = event else { return };
                if event.paths.iter().any(|path| path == &callback_file) {
                    let _ = output.try_send(Message::VHosts(VHostsMessage::ConfigFileChanged));
                }
            });

        let Ok(mut watcher) = watcher_result else {
            return;
        };

        if watcher
            .watch(&watch_dir, RecursiveMode::NonRecursive)
            .is_err()
        {
            return;
        }

        iced::futures::future::pending::<()>().await;
    })
}
