/*
 * Copyright (c) 2024, MLC 'Strawmelonjuice' Bloeiman
 *
 * Licensed under the GNU AFFERO GENERAL PUBLIC LICENSE Version 3, see the LICENSE file for more information.
 */
import * as CynthiaPluginAPI from "cynthia-plugin-api/main";
import { Cynthia } from "cynthia-plugin-api/main";
import * as process from "node:process";
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

import {
  terminalOut as console,
  CynthiaPassed,
} from "../../node-plugin-api/main";
import * as handlebars from "handlebars";
import * as fs from "node:fs";
import path from "node:path";
import {
  Plugincompat,
  PluginPackageJson,
  type PluginBase,
} from "./types/internal_plugins";
Cynthia.console.debug(`Starting in cwd: ${process.cwd()}`);
Cynthia.console.info("Config loaded.");
Cynthia.console.debug(`Config: ${JSON.stringify(config)}`);
Cynthia.console.info(
  `External Javascript Runtime Server starting in: ${process.argv0}`,
);
const cynthiabase: PluginBase = {
  modifyOutputHTML: [
    (htmlin: string, Cynthia: typeof CynthiaPluginAPI.CynthiaPassed) => {
      // Make no changes. Return unchanged.
      return htmlin;
    },
  ],
  modifyBodyHTML: [
    (htmlin: string, Cynthia: typeof CynthiaPluginAPI.CynthiaPassed) => {
      // Return with a little comment.
      return `${htmlin}\n<!-- test... Body modifier Node plugins enabled! -->\n`;
    },
  ],
  requestOptions: [
    (
      WebRequest: CynthiaPluginAPI.IncomingWebRequest,
      Cynthia: typeof CynthiaPluginAPI.CynthiaWebResponderApi,
    ) => {
      // Make no changes. Return unchanged.
      // This function doesn't actually return. It just sends out `Cynthia.answer(() => { return response });` if capturing.
    },
  ],
};

for (const pluginname in config.plugins) {
  if (config.plugins[pluginname].plugin_enabled === false) {
    continue;
  }
  if (
    config.plugins[pluginname].plugin_runtime === "javascript" ||
    config.plugins[pluginname].plugin_runtime === "js"
  ) {
    const plugin = (() => {
      const pluginPackageJson: PluginPackageJson = require(
        path.join(
          process.cwd(),
          "cynthiaPlugins/",
          pluginname,
          "/package.json",
        ),
      );
      if (pluginPackageJson["cynthia-plugin-compat"] !== Plugincompat) {
        Cynthia.console.error(
          `Plugin ${pluginname} is not compatible with this version of Cynthia.`,
        );
        return;
      }
      const pluginEntryJs = path.join(
        process.cwd(),
        "cynthiaPlugins/",
        pluginname,
        pluginPackageJson["cynthia-plugin"],
      );
      return require(pluginEntryJs);
    })();
    if (typeof plugin.modifyOutputHTML === "function") {
      cynthiabase.modifyOutputHTML.push(plugin.modifyOutputHTML);
    }
    if (typeof plugin.requestOptions === "function") {
      cynthiabase.requestOptions.push(plugin.expressActions);
    }
    if (typeof plugin.modifyBodyHTML === "function") {
      cynthiabase.modifyBodyHTML.push(plugin.modifyBodyHTML);
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
setInterval(clean, 300000);
clean();
// Warn Deno users that the forced garbage collection is unavailable.
if (process.argv0.includes("deno")) {
  Cynthia.console.warn(
    "Forced garbage collection unavailable in Deno. Instead Deno's own 'predictable' garbage collection is used.",
  );
}
process.stdin.resume();
process.stdin.on("data", handle);

async function handle(buffer: Buffer) {
  if (buffer.toString().startsWith("parse: ")) {
    console.debug("Got a request.");
    console.debug(`Buffer: ${buffer}`);
    const requestAsString = buffer.toString().replace("parse: ", "");
    const request: CynthiaPluginAPI.GenericRequest =
      JSON.parse(requestAsString);
    switch (request.body.for) {
      case "Exit": {
        console.error("Exiting...");
        return process.exit(0);
      }
      case "WebRequest": {
        console.debug("Got a WebRequest.");
        console.debug(`Request: ${requestAsString}`);
        // Until this is implemented, we will just return EmptyOk.
        const response = new CynthiaPluginAPI.EmptyOKResponse(request.id);
        return Cynthia.send(response);
      }
      case "PostlistRenderRequest": {
        try {
          // streq helper
          // This helper checks if two strings are equal.
          // Usage: {{#if (streq postid "sasfs")}} ... {{/if}}
          handlebars.registerHelper("streq", (a: string, b: string) => a === b);

          const request: CynthiaPluginAPI.PostlistRenderRequest =
            JSON.parse(requestAsString);
          const template = fs.readFileSync(request.body.template_path, "utf8");
          const compiled = handlebars.compile(template);
          const html = compiled(request.body.template_data);
          let page = html;
          cynthiabase.modifyBodyHTML.forEach((modifier) => {
            page = modifier(page, CynthiaPassed);
          });
          const response = new CynthiaPluginAPI.OkStringResponse(
            request.id,
            page,
          );
          return Cynthia.send(response);
        } catch (e) {
          console.error(e);
          const response = new CynthiaPluginAPI.ErrorResponse(request.id, "");
          return Cynthia.send(response);
        }
      }
      case "ContentRenderRequest": {
        try {
          // streq helper
          // This helper checks if two strings are equal.
          // Usage: {{#if (streq postid "sasfs")}} ... {{/if}}
          handlebars.registerHelper("streq", (a: string, b: string) => a === b);

          const request: CynthiaPluginAPI.ContentRenderRequest =
            JSON.parse(requestAsString);
          const template = fs.readFileSync(request.body.template_path, "utf8");
          const compiled = handlebars.compile(template);
          const html = compiled(request.body.template_data);
          let page = html;
          cynthiabase.modifyBodyHTML.forEach((modifier) => {
            page = modifier(page, CynthiaPassed);
          });
          const response = new CynthiaPluginAPI.OkStringResponse(
            request.id,
            page,
          );
          return Cynthia.send(response);
        } catch (e) {
          console.error(e);
          const response = new CynthiaPluginAPI.ErrorResponse(request.id, "");
          return Cynthia.send(response);
        }
      }
      case "Test": {
        const request: CynthiaPluginAPI.TestRequest =
          JSON.parse(requestAsString);
        // {"id":0,"body":{"as":"OkString","value":"Yes."}}
        const response = new CynthiaPluginAPI.OkStringResponse(
          request.id,
          `Successfully received test request. Test passed with echo: "${request.body.test}"`,
        );
        return Cynthia.send(response);
      }
    }
  } else {
    console.log(`Got: ${buffer}`);
  }
}
