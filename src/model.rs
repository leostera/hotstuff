use std::path::PathBuf;

use crate::parser::{parse_sexp, SExpr};

const SITEFILE_NAME: &str = "site";

#[derive(Debug, Clone, Default)]
pub struct Project {
    root: PathBuf,
    output_dir: PathBuf,
}

impl Project {
    pub fn output_dir(self) -> PathBuf {
        self.output_dir
    }
    pub fn root(self) -> PathBuf {
        self.root
    }

    pub fn new() -> Project {
        Project {
            root: PathBuf::from("."),
            output_dir: PathBuf::from("./_public"),
        }
    }

    pub fn with_root(self, root: PathBuf) -> Project {
        Project { root, ..self }
    }

    pub fn with_output_dir(self, output_dir: PathBuf) -> Project {
        Project { output_dir, ..self }
    }
}

#[derive(Debug, Clone)]
pub struct Sitefile {
    dir: PathBuf,
    template: Option<PathBuf>,
    assets: Option<Vec<PathBuf>>,
}

impl Sitefile {
    pub fn assets(self) -> Option<Vec<PathBuf>> {
        self.assets
    }
    pub fn dir(self) -> PathBuf {
        self.dir
    }
    pub fn template(self) -> Option<PathBuf> {
        self.template
    }

    pub fn name() -> String {
        SITEFILE_NAME.to_string()
    }

    pub fn from_dir_path(root: PathBuf) -> Option<Sitefile> {
        let site_path = root.clone().join(Sitefile::name());
        if let Ok(file) = std::fs::read_to_string(site_path) {
            let mut sitefile = Sitefile {
                dir: root,
                template: None,
                assets: None,
            };

            for sexp in parse_sexp(&file) {
                if let SExpr::List(sexp) = &sexp {
                    let name = sexp[0].clone();
                    if name == SExpr::Atom("assets".to_string()) {
                        sitefile.assets = Some(
                            sexp[1..]
                                .iter()
                                .map(|a| {
                                    if let SExpr::Atom(a) = a {
                                        a.to_string()
                                    } else {
                                        "".to_string()
                                    }
                                    .replace("\"", "")
                                    .replace("./", "")
                                })
                                .map(PathBuf::from)
                                .collect(),
                        )
                    }

                    if name == SExpr::Atom("template".to_string()) {
                        let template_name = if let SExpr::Atom(a) = &sexp[1] {
                            a.to_string()
                        } else {
                            "".to_string()
                        }
                        .replace("\"", "")
                        .replace("./", "");
                        sitefile.template = Some(PathBuf::from(template_name));
                    }
                }
            }

            Some(sitefile)
        } else {
            None
        }
    }
}
