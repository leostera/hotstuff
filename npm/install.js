const os = require("os");
const https = require("https");
const tar = require("tar");
const { join } = require("path");

const error = (msg) => {
  console.error(msg);
  process.exit(1);
};

const { version, name, repository } = require("./package.json");

const [scope, package_name] = name.split("/");

const supportedPlatforms = [
  {
    TYPE: "Windows_NT",
    ARCHITECTURE: "x64",
    RUST_TARGET: "x86_64-pc-windows-msvc",
  },
  {
    TYPE: "Linux",
    ARCHITECTURE: "x64",
    RUST_TARGET: "x86_64-unknown-linux-musl",
  },
  {
    TYPE: "Darwin",
    ARCHITECTURE: "x64",
    RUST_TARGET: "x86_64-apple-darwin",
  },
];

const getPlatform = () => {
  const type = os.type();
  const architecture = os.arch();

  for (let index in supportedPlatforms) {
    let supportedPlatform = supportedPlatforms[index];
    if (
      type === supportedPlatform.TYPE &&
      architecture === supportedPlatform.ARCHITECTURE
    ) {
      return supportedPlatform.RUST_TARGET;
    }
  }

  error(
    `Platform with type "${type}" and architecture "${architecture}" is not supported by ${name}.\nYour system must be one of the following:\n\n${[
      "TYPE\tARCHTRUST_TARGET",
      "================================",
      ...supportedPlatforms.map(
        ({ TYPE, ARCHITECTURE, RUST_TARGET }) =>
          `${TYPE}\t${ARCHITECTURE}\t${RUST_TARGET}`
      ),
    ].join("\n")}`
  );
};

const platform = getPlatform();
const url = `${repository.url}/releases/download/v${version}/${package_name}-v${version}-${platform}.tar.gz`;
console.log(`Downloading release from:\n  ${url}`);

console.log("From:", process.cwd());

const install = (archive) => {
  archive
    .pipe(tar.extract({ strict: true, unlink: true }))
    .on("error", (e) => error(`Error fetching release: ${e.message}`))
    .on("finish", () => console.log(`${package_name} has been installed!`));
};

https.get(url, (res) => {
  if (res.statusCode >= 300 && res.statusCode < 400 && res.headers.location) {
    https.get(res.headers.location, (res) => {
      install(res);
    });
  }

  if (res.statusCode == 404) {
    console.log("We could not find this release yet. Are you sure you got the version right?");
  }
});
