/*
 * Copyright (c) 2024, MLC 'Strawmelonjuice' Bloeiman
 *
 * Licensed under the GNU AFFERO GENERAL PUBLIC LICENSE Version 3, see the LICENSE file for more information.
 */

import * as process from "node:process";

import {
  Cynthia,
  CynthiaPassed,
  type CynthiaPlugin,
} from "cynthia-plugin-api/main";
import type Config from "./types/config";
const config: Config = (() => {
  let conf = "";
  for (let i = 0; i < process.argv.length; i++) {
    if (process.argv[i] === "--config") {
      conf = process.argv[i + 1];
    }
  }
  return JSON.parse(conf);
})();

import path from "node:path";
import {
  type PluginBase,
  type PluginPackageJson,
  Plugincompat,
  newPluginBase,
} from "./types/internal_plugins";
import handle from "./handler";
Cynthia.console.debug(`Starting in cwd: ${process.cwd()}`);
Cynthia.console.info("Config loaded.");
Cynthia.console.debug(`Config: ${JSON.stringify(config)}`);
Cynthia.console.info(
  `External Javascript Runtime Server starting in: ${process.argv0}`,
);
const cynthiaPluginFoundation: PluginBase = newPluginBase;

for (const pluginIndex in config.plugins) {
  if (!config.plugins[pluginIndex].plugin_enabled) {
    continue;
  }
  if (
    config.plugins[pluginIndex].plugin_runtime === "javascript" ||
    config.plugins[pluginIndex].plugin_runtime === "js"
  ) {
    const pluginName = config.plugins[pluginIndex].plugin_name;
    Cynthia.console.info(`Loading plugin ${pluginName}...`);
    try {
      const plugin: CynthiaPlugin = (() => {
        const pluginPackageJson: PluginPackageJson = require(
          path.join(
            process.cwd(),
            "cynthiaPlugins/",
            pluginName,
            "/package.json",
          ),
        );
        if (pluginPackageJson["cynthia-plugin-compat"] !== Plugincompat) {
          throw new Error(
            `Plugin ${pluginName} is not compatible with this version of Cynthia.`,
          );
        }
        const pluginDir = path.join(
          process.cwd(),
          "cynthiaPlugins/",
          pluginName,
        );
        const exec = require("child_process").execSync;
        const runner = (() => {
          if (process.argv0.includes("bun")) {
            return [
              process.argv0 + " --bun --silent",
              process.argv0 + " --silent",
            ];
          }
          return ["npm", "npm"];
        })();
        Cynthia.console.info(`Running: '${runner[1]} install'`);
        try {
          const stdout = exec(`${runner[1]} install`, {
            cwd: pluginDir,
          });

          Cynthia.console.debug(stdout);
        } catch (error: unknown) {
          Cynthia.console.error(
            `Error installing dependencies for ${pluginName}: ${error}`,
          );
          return;
        }

        // Now we gotta run the plugin's prerun script. (onBeforeRun)
        if (pluginPackageJson.scripts.onBeforeRun) {
          Cynthia.console.info(`Running: '${runner[0]} run onBeforeRun'`);
          try {
            const stdout = exec(`${runner[0]} run onBeforeRun`, {
              cwd: pluginDir,
            });
            Cynthia.console.debug(stdout);
          } catch (error) {
            Cynthia.console.error(
              `Error running onBeforeRun script for plugin ${pluginName}: ${error}`,
            );
            return;
          }
        }
        const pluginEntryJs = path.join(
          pluginDir,
          pluginPackageJson["cynthia-plugin"],
        );
        return require(pluginEntryJs);
      })();
      if (typeof plugin.modifyResponseHTML === "function") {
        cynthiaPluginFoundation.modifyResponseHTML.push(
          plugin.modifyResponseHTML,
        );
      }
      if (typeof plugin.modifyRequest === "function") {
        cynthiaPluginFoundation.modifyRequest.push(plugin.modifyRequest);
      }
      if (typeof plugin.modifyResponseHTMLBodyFragment === "function") {
        cynthiaPluginFoundation.modifyResponseHTMLBodyFragment.push(
          plugin.modifyResponseHTMLBodyFragment,
        );
      }
      if (typeof plugin.onClearInterval === "function") {
        cynthiaPluginFoundation.onClearInterval.push(plugin.onClearInterval);
      }
      if (typeof plugin.onLoad === "function") {
        plugin.onLoad(CynthiaPassed);
      }
    } catch (e) {
      Cynthia.console.error(`Error loading plugin ${pluginName}: ${e}`);
    }
  }
}

function clean() {
  switch (true) {
    case process.argv0.includes("node"):
      if (global.gc) {
        Cynthia.console.debug("Forcing garbage collection.");
        try {
          global.gc();
        } catch (e) {
          Cynthia.console.error(`Forcing garbage collection failed:  ${e}`);
        }
      } else {
        Cynthia.console.error(
          "Forced garbage collection unavailable.  Pass --expose-gc " +
            "when launching node to enable forced garbage collection.",
        );
      }
      break;
    case process.argv0.includes("deno"):
      break;
    case process.argv0.includes("bun"):
      Cynthia.console.debug("Forcing garbage collection.");
      try {
        Bun.gc(false);
      } catch (e) {
        Cynthia.console.error(`Forcing garbage collection failed:  ${e}`);
      }
      {
      }
      break;
  }
}
function cleanInterval() {
  for (const fn of cynthiaPluginFoundation.onClearInterval) {
    fn(CynthiaPassed);
  }
  clean();
}
setInterval(cleanInterval, 300000);
cleanInterval();
// Warn Deno users that the forced garbage collection is unavailable.
if (process.argv0.includes("deno")) {
  Cynthia.console.warn(
    "Forced garbage collection unavailable in Deno. Instead Deno's own 'predictable' garbage collection is used.",
  );
}
process.stdin.resume();
process.stdin.on("data", async (s) => {
  await handle(s, cynthiaPluginFoundation);
});
