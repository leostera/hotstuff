use std::fs;
use std::path::PathBuf;

use crate::build_rules::CompilationUnit;
use crate::model::{Project, Sitefile};

#[derive(Debug, Clone)]
pub enum BuildPlan {
    Node(CompilationUnit, Vec<BuildPlan>),
    Leaf(CompilationUnit),
}

impl BuildPlan {
    pub fn start_with(cunit: CompilationUnit) -> BuildPlan {
        BuildPlan::Leaf(cunit)
    }

    pub fn and_then(self, tasks: Vec<BuildPlan>) -> BuildPlan {
        match self {
            BuildPlan::Leaf(cunit) => BuildPlan::Node(cunit, tasks),
            BuildPlan::Node(cunit, _) => BuildPlan::Node(cunit, tasks),
        }
    }

    pub fn deps(self) -> Vec<BuildPlan> {
        match self {
            BuildPlan::Leaf(_) => vec![],
            BuildPlan::Node(_, deps) => deps,
        }
    }

    pub fn map<F>(self, f: F) -> BuildPlan
    where
        F: Copy + FnOnce(CompilationUnit) -> CompilationUnit,
    {
        match self {
            BuildPlan::Node(cunit, deps) => {
                let new_cunit = f(cunit.clone());
                if new_cunit == cunit {
                    BuildPlan::Node(new_cunit, deps)
                } else {
                    BuildPlan::Node(new_cunit, deps.into_iter().map(move |d| d.map(f)).collect())
                }
            }
            BuildPlan::Leaf(cunit) => BuildPlan::Leaf(f(cunit)),
        }
    }

    pub fn breadth_first_iter<'a>(&'a self) -> Box<dyn Iterator<Item = &CompilationUnit> + 'a> {
        match self {
            BuildPlan::Leaf(cunit) => Box::new(vec![cunit].into_iter()),
            BuildPlan::Node(cunit, deps) => {
                let root = vec![cunit].into_iter();
                let deps = deps.iter().flat_map(BuildPlan::breadth_first_iter);
                Box::new(root.chain(deps))
            }
        }
    }
}

fn plan_site(root: PathBuf, output_dir: PathBuf, files: &Vec<PathBuf>) -> Option<BuildPlan> {
    let site = Sitefile::from_dir_path(root.clone())?;
    let (docs, _): (Vec<String>, Vec<String>) = files
        .iter()
        .map(|p| p.file_name().unwrap())
        .map(|p| String::from(p.to_str().unwrap()))
        .partition(|p| p.ends_with("html") || p.ends_with("md"));

    let template = site.clone().template();

    let assets: Vec<BuildPlan> = site
        .assets()
        .unwrap_or_else(|| vec![])
        .into_iter()
        .flat_map(|p| {
            if p.to_str().unwrap().eq(".") {
                fs::read_dir(root.clone())
                    .expect("Could not read root folder")
                    .map(|e| e.unwrap().path())
                    .filter(|p| !p.is_dir())
                    .map(|p| p.file_name().unwrap().to_str().unwrap().to_string())
                    .map(PathBuf::from)
                    .collect()
            } else {
                vec![p]
            }
        })
        .filter(|p| {
            !p.ends_with("hotstuff-project")
                && !p.ends_with("site")
                && !p.ends_with("swp")
                && !p.ends_with("swo")
                && !p.ends_with("~")
        })
        .map(|a| CompilationUnit::Copy {
            input: root.clone().join(a.clone()),
            output: output_dir.clone().join(a),
        })
        .map(BuildPlan::start_with)
        .collect();

    let docs = docs
        .into_iter()
        .filter(|d| {
            if let Some(template) = &template {
                !PathBuf::from(d).eq(template)
            } else {
                true
            }
        })
        .map(|d| {
            let output = output_dir.clone().join(d.clone()).with_extension("html");
            let cunit = CompilationUnit::Compile {
                input: root.clone().join(d),
                output: output.clone(),
            };
            let compile = BuildPlan::start_with(cunit);

            match &template {
                Some(template) => {
                    let cunit = CompilationUnit::Template {
                        input: output.clone(),
                        output,
                        template: root.clone().join(template),
                    };
                    let template = vec![BuildPlan::start_with(cunit)];
                    compile.and_then(template)
                }
                None => compile,
            }
        })
        .collect::<Vec<BuildPlan>>();

    let create_dir = CompilationUnit::CreateDir {
        path: output_dir.clone(),
    };
    let mut copy_and_compile_docs = vec![];

    if let Some(template) = &template {
        let copy_template = CompilationUnit::Copy {
            input: root.join(template),
            output: output_dir.join(template),
        };
        copy_and_compile_docs.push(BuildPlan::start_with(copy_template).and_then(docs))
    } else {
        for doc in docs {
            copy_and_compile_docs.push(doc)
        }
    };

    for asset in assets {
        copy_and_compile_docs.push(asset)
    }

    Some(BuildPlan::start_with(create_dir).and_then(copy_and_compile_docs))
}

fn find_sites(root: PathBuf, output_dir: PathBuf) -> Vec<BuildPlan> {
    if root.is_dir() {
        let (files, dirs) = fs::read_dir(root.clone())
            .unwrap()
            .map(|e| e.unwrap().path())
            .partition(|p| !p.is_dir());

        let root_graph = plan_site(root, output_dir.clone(), &files);

        let mut subsites: Vec<BuildPlan> = dirs
            .into_iter()
            .filter(|subroot| !subroot.eq(&output_dir.clone()))
            .flat_map(|subroot| {
                find_sites(
                    subroot.clone(),
                    output_dir.clone().join(subroot.file_name().unwrap()),
                )
            })
            .collect();

        if let Some(rg) = root_graph {
            subsites.push(rg);
        }

        subsites
    } else {
        vec![]
    }
}

pub fn plan_build(project: Project) -> BuildPlan {
    let create_dir = CompilationUnit::CreateDir {
        path: project.clone().output_dir(),
    };
    let build_sites = find_sites(project.clone().root(), project.output_dir());
    BuildPlan::start_with(create_dir).and_then(build_sites)
}
