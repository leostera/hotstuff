use log::{debug, info};

use crate::build_graph::BuildPlan;
use crate::build_rules::{compile_unit, Artifact, CompilationUnit};

fn mstat(path: std::path::PathBuf) -> std::time::SystemTime {
    if let Ok(meta) = std::fs::metadata(path) {
        meta.modified().unwrap()
    } else {
        std::time::SystemTime::UNIX_EPOCH
    }
}

impl BuildPlan {
    pub fn compute_diff(self) -> BuildPlan {
        self.map(|cunit| match cunit {
            CompilationUnit::CreateDir { path } => {
                let unit = CompilationUnit::CreateDir { path: path.clone() };
                if std::fs::canonicalize(path).is_ok() {
                    CompilationUnit::CacheHit {
                        unit: Box::new(unit),
                    }
                } else {
                    unit
                }
            }

            CompilationUnit::Copy { input, output } => {
                let unit = CompilationUnit::Copy {
                    input: input.clone(),
                    output: output.clone(),
                };
                if mstat(input) >= mstat(output) {
                    unit
                } else {
                    CompilationUnit::CacheHit {
                        unit: Box::new(unit),
                    }
                }
            }

            CompilationUnit::Compile { input, output } => {
                let unit = CompilationUnit::Compile {
                    input: input.clone(),
                    output: output.clone(),
                };
                if mstat(input) >= mstat(output) {
                    unit
                } else {
                    CompilationUnit::CacheHit {
                        unit: Box::new(unit),
                    }
                }
            }

            CompilationUnit::Template {
                input,
                output,
                template,
            } => {
                let unit = CompilationUnit::Template {
                    input: input.clone(),
                    output: output.clone(),
                    template: template.clone(),
                };
                let mstat_output = mstat(output);
                if mstat(input) > mstat_output || mstat(template) > mstat_output {
                    unit
                } else {
                    CompilationUnit::CacheHit {
                        unit: Box::new(unit),
                    }
                }
            }

            hit @ CompilationUnit::CacheHit { .. } => hit,
        })
    }

    pub fn execute(self) -> Vec<Artifact> {
        let t0 = std::time::Instant::now();
        let mut artifacts = vec![];
        for cunit in self.breadth_first_iter() {
            match cunit {
                hit @ CompilationUnit::CacheHit { .. } => debug!("\x1b[90m{:?}\x1b[0m", hit),
                unit => {
                    info!("\x1b[94m{:?}\x1b[0m", unit.clone());
                    let artifact = compile_unit(unit.clone())
                        .unwrap_or_else(|_| panic!("Could not complete task: {:?}", unit));
                    artifacts.push(artifact);
                }
            }
        }
        if !artifacts.is_empty() {
            info!(
                "Built {} artifacts in {}ms",
                artifacts.len(),
                t0.elapsed().as_millis()
            );
        }
        artifacts
    }
}
