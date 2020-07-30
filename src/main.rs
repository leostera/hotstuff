use fern::colors::{Color, ColoredLevelConfig};
use log::info;
use std::path::PathBuf;
use structopt::StructOpt;

use hotstuff::build_graph;
use hotstuff::http_server;
use hotstuff::model;

#[derive(StructOpt, Debug, Clone)]
#[structopt(
    name = "hotstuff",
    setting = structopt::clap::AppSettings::ColoredHelp,
    about = "
hotstuff is a composable no-nonsense static site generator.

It does 2 things:

  * `hotstuff build` - incrementally compile a file tree of documents with assets
  * `hotstuff serve` - serve them with live-reload over HTTP for local development

It keeps no in-memory state, so it has **crazy fast cold starts**.
    "
)]
struct HotStuff {
    #[structopt(short = "v", long = "verbose", help = "turn on verbosity")]
    verbose: bool,

    #[structopt(subcommand)]
    cmd: Goal,
}

impl HotStuff {
    async fn run(self) {
        self.setup_logging();
        self.cmd.run().await;
    }

    fn setup_logging(&self) {
        let colors_line = ColoredLevelConfig::new()
            .error(Color::Red)
            .warn(Color::Yellow)
            .info(Color::White)
            .debug(Color::White)
            .trace(Color::BrightBlack);
        let colors_level = colors_line.clone().info(Color::Green);
        fern::Dispatch::new()
            .format(move |out, message, record| {
                out.finish(format_args!(
                    "{color_line}{date} {level}{color_line} :: {message}\x1B[0m",
                    color_line = format_args!(
                        "\x1B[{}m",
                        colors_line.get_color(&record.level()).to_fg_str()
                    ),
                    date = chrono::Local::now().format("%H:%M:%S"),
                    level = colors_level.color(record.level()),
                    message = message,
                ));
            })
            .level(if self.verbose {
                log::LevelFilter::Debug
            } else {
                log::LevelFilter::Info
            })
            .level_for("pretty_colored", log::LevelFilter::Trace)
            .chain(std::io::stdout())
            .apply()
            .unwrap();
    }
}

#[derive(StructOpt, Debug, Clone)]
enum Goal {
    Build(BuildOpt),

    Serve(ServeOpt),
}

impl Goal {
    async fn run(self) {
        match self {
            Goal::Build(opts) => opts.build().await,
            Goal::Serve(opts) => opts.serve().await,
        }
    }
}

#[derive(StructOpt, Debug, Clone)]
#[structopt(name = "serve", about = "serve this project with livereloading")]
struct ServeOpt {
    #[structopt(
        short = "p",
        long = "port",
        name = "PORT",
        default_value = "4000",
        help = "the port in which to bind the server"
    )]
    port: u16,

    #[structopt(
        short = "r",
        long = "root",
        name = "ROOT",
        default_value = "./",
        help = "the root folder from which to serve files",
        parse(from_os_str)
    )]
    root: PathBuf,

    #[structopt(
        short = "o",
        long = "output",
        name = "OUTPUT",
        default_value = "./_public",
        help = "the folder where to place the compiled sites",
        parse(from_os_str)
    )]
    output_dir: PathBuf,
}

impl ServeOpt {
    async fn serve(self) {
        let project = model::Project::new()
            .with_root(self.root)
            .with_output_dir(self.output_dir);

        build_graph::plan_build(project.clone())
            .compute_diff()
            .execute();

        http_server::Server::from_project(project)
            .with_port(self.port)
            .listen()
            .await;
    }
}

#[derive(StructOpt, Debug, Clone)]
#[structopt(name = "build", about = "build this project")]
struct BuildOpt {
    #[structopt(
        short = "r",
        long = "root",
        name = "ROOT",
        default_value = "./",
        help = "the root folder from which to serve files",
        parse(from_os_str)
    )]
    root: PathBuf,

    #[structopt(
        short = "o",
        long = "output",
        name = "OUTPUT",
        default_value = "./_public",
        help = "the folder where to place the compiled sites",
        parse(from_os_str)
    )]
    output_dir: PathBuf,

    #[structopt(short = "f", long = "force", help = "execute all compilation units")]
    force: bool,
}

impl BuildOpt {
    async fn build(self) {
        let t0 = std::time::Instant::now();
        info!("Building project...");
        let project = model::Project::new()
            .with_root(self.root)
            .with_output_dir(self.output_dir);

        let build_plan = {
            let build_plan = build_graph::plan_build(project);

            if self.force {
                build_plan
            } else {
                build_plan.compute_diff()
            }
        };

        let _artifacts = build_plan.execute();
        info!("Done in {}ms", t0.elapsed().as_millis());
    }
}

#[tokio::main]
async fn main() {
    HotStuff::from_args().run().await;
}
