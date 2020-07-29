/*
use crate::model::Project;

use notify::Watcher as OtherWatcher;
use std::collections::HashSet;
use std::sync::{Arc, RwLock};

pub struct Watcher {
    project: Project,
    inner_watcher: Option<notify::RecommendedWatcher>,
    changes: Arc<RwLock<HashSet<String>>>,
}

impl Watcher {
    pub fn changes(self) -> Arc<RwLock<HashSet<String>>> {
        self.changes.clone()
    }

    pub fn from_project(project: Project) -> Watcher {
        Watcher {
            project,
            inner_watcher: None,
            changes: Arc::new(RwLock::new(HashSet::new())),
        }
    }

    pub fn start(mut self) -> Watcher {
        if let Some(_) = self.inner_watcher {
            return self;
        }

        let root = std::fs::canonicalize(self.project.clone().root().clone())
            .expect("Root directory for watching does not exist");
        let changes = self.changes.clone();
        let mut watcher: notify::RecommendedWatcher = notify::Watcher::new_immediate(move |res| {
            println!("Files changed! {:?}", res);

            match res {
                Ok(notify::event::Event {
                    kind: _kind,
                    paths,
                    attrs: _attrs,
                }) => {
                    let paths: Vec<String> = paths
                        .iter()
                        .filter(|p| !p.is_dir())
                        .map(|p| p.to_str().unwrap())
                        .filter(|p| !p.ends_with("~"))
                        .filter(|p| !p.ends_with("swp"))
                        .filter(|p| !p.ends_with("tmp"))
                        .filter(|p| !p.starts_with("/."))
                        .map(String::from)
                        .collect();

                    loop {
                        match changes.try_write() {
                            Ok(mut cfs) => {
                                for path in paths.clone() {
                                    cfs.insert(path);
                                }
                                break;
                            }
                            Err(_) => (),
                        }
                    }
                }
                _ => (),
            }
        })
        .expect("Could not create watcher");

        watcher
            .watch(root.clone(), notify::RecursiveMode::Recursive)
            .expect("Could not configure watcher recursively")

        println!("Watching for changes in {:?}", root);

        self.inner_watcher = Some(watcher);

        self
    }
}
*/
