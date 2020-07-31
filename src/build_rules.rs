use nipper::Document;
use std::ffi::OsStr;
use std::path::PathBuf;

#[derive(Debug, Clone, PartialEq)]
pub enum CompilationUnit {
    CreateDir {
        path: PathBuf,
    },

    CacheHit {
        unit: Box<CompilationUnit>,
    },

    Copy {
        input: PathBuf,
        output: PathBuf,
    },

    Compile {
        input: PathBuf,
        output: PathBuf,
    },

    Template {
        input: PathBuf,
        output: PathBuf,
        template: PathBuf,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub enum Artifact {
    File(PathBuf),
    Nothing,
}

pub fn compile_unit(cunit: CompilationUnit) -> Result<Artifact, impl std::error::Error> {
    match cunit {
        CompilationUnit::CreateDir { path } => {
            let path_str = path.to_str().expect("Could not access path");
            std::fs::create_dir_all(path_str).map(|_| Artifact::File(path))
        }

        CompilationUnit::CacheHit { .. } => Ok(Artifact::Nothing),

        CompilationUnit::Copy { input, output } => {
            std::fs::copy(input, output.clone()).map(|_| Artifact::File(output))
        }

        CompilationUnit::Compile { input, output } => {
            let raw = std::fs::read_to_string(input.clone())?;
            let ext = input.extension().and_then(OsStr::to_str).unwrap_or("");
            let compiled = match ext {
                "md" => comrak::markdown_to_html(&raw, &comrak::ComrakOptions::default()),
                _ => raw,
            };
            std::fs::write(output.clone(), compiled).map(|_| Artifact::File(output))
        }

        CompilationUnit::Template {
            input,
            output,
            template,
        } => {
            let raw = std::fs::read_to_string(input)?;
            let html = Document::from(&raw);
            let title = html.select("h1").text();
            let template = std::fs::read_to_string(template)?;
            let compiled = template
                .replace("{| title |}", &title)
                .replace("{| document |}", &raw);
            std::fs::write(output.clone(), compiled).map(|_| Artifact::File(output))
        }
    }
}
