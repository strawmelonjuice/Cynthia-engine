/*
 * Copyright (c) 2024, MLC 'Strawmelonjuice' Bloeiman
 *
 * Licensed under the GNU AFFERO GENERAL PUBLIC LICENSE Version 3, see the LICENSE file for more information.
 */

import * as process from "node:process";

import {
  Cynthia,
  type CynthiaPassed,
  type CynthiaPlugin,
  type CynthiaWebResponderApi,
  type IncomingWebRequest,
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

for (const pluginname in config.plugins) {
  if (config.plugins[pluginname].plugin_enabled === false) {
    continue;
  }
  if (
    config.plugins[pluginname].plugin_runtime === "javascript" ||
    config.plugins[pluginname].plugin_runtime === "js"
  ) {
    try {
      const plugin: CynthiaPlugin = (() => {
        const pluginPackageJson: PluginPackageJson = require(
          path.join(
            process.cwd(),
            "cynthiaPlugins/",
            pluginname,
            "/package.json",
          ),
        );
        if (pluginPackageJson["cynthia-plugin-compat"] !== Plugincompat) {
          throw new Error(
            `Plugin ${pluginname} is not compatible with this version of Cynthia.`,
          );
        }
        const pluginEntryJs = path.join(
          process.cwd(),
          "cynthiaPlugins/",
          pluginname,
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
    } catch (e) {
      Cynthia.console.error(`Error loading plugin ${pluginname}: ${e}`);
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
    fn();
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
  handle(s, cynthiaPluginFoundation);
});
