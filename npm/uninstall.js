#!/usr/bin/env node

const { accessSync, unlinkSync } = require("fs");

const { version, name, repository } = require("./package.json");
const [scope, package_name] = name.split("/");

const path = `./node_modules/${name}/bin/${package_name}`;

try {
  accessSync(path);
  unlinkSync(`/usr/local/bin/${package_name}`);
  console.log(`${package_name} successfully uninstalled.`);
} catch (_) {
  console.log(`${package_name} not installed yet. Moving on...`);
}
