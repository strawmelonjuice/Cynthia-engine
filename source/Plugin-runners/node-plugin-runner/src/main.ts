/*
 * Copyright (c) 2024, MLC 'Strawmelonjuice' Bloeiman
 *
 * Licensed under the GNU AFFERO GENERAL PUBLIC LICENSE Version 3, see the LICENSE file for more information.
 */

import * as CynthiaPluginAPI from "cynthia-plugin-api/main";
import { Cynthia } from "cynthia-plugin-api/main";
import * as process from "node:process";

import { terminalOut as console } from "../../node-plugin-api/main";
import * as handlebars from "handlebars";
import * as fs from "node:fs";

console.info("Node plugin server starting in " + process.argv0);
const cynthiabase = {
  modifyOutputHTML: [
    (htmlin: string, Cynthia: typeof CynthiaPluginAPI.CynthiaPassed) => {
      // Make no changes. Return unchanged.
      return htmlin;
    },
  ],
  modifyBodyHTML: [
    (htmlin: string, Cynthia: typeof CynthiaPluginAPI.CynthiaPassed) => {
      // Make no changes. Return unchanged.
      return htmlin;
    },
  ],
  requestOptions: [
    (
      WebRequest: CynthiaPluginAPI.Incoming.WebRequest,
      Cynthia: typeof CynthiaPluginAPI.CynthiaWebResponderApi,
    ) => {
      // Make no changes. Return unchanged.
      // This function doesn't actually return. It just sends out `Cynthia.answer(() => { return response });` if capturing.
    },
  ],
};
process.stdin.resume();
process.stdin.on("data", handle);

async function handle(buffer: Buffer) {
  if (buffer.toString().startsWith("parse: ")) {
    console.debug("Got a request.");
    console.debug(`Buffer: ${buffer}`);
    const str = buffer.toString().replace("parse: ", "");
    let request: CynthiaPluginAPI.GenericRequest = JSON.parse(str);
    switch (request.body.for) {
      case "Exit": {
        console.error("Exiting...");
        return process.exit(0);
      }
      case "WebRequest": {
        // Until this is implemented, we will just return EmptyOk.
        let response = new CynthiaPluginAPI.EmptyOKResponse(request.id);
        return Cynthia.send(response);
      }
      case "ContentRenderRequest": {
        try {
          // streq helper
          // This helper checks if two strings are equal.
          // Usage: {{#if (streq postid "sasfs")}} ... {{/if}}
          handlebars.registerHelper("streq", function (a: string, b: string) {
            return a === b;
          });

          let request: CynthiaPluginAPI.ContentRenderRequest = JSON.parse(str);
          const template = fs.readFileSync(request.body.template_path, "utf8");
          const compiled = handlebars.compile(template);
          const html = compiled(request.body.template_data);
          let page = html;
          cynthiabase.modifyBodyHTML.forEach((modifier) => {
            page = modifier(page /*, Cynthia*/);
          });
          const response = new CynthiaPluginAPI.OkStringResponse(
            request.id,
            html,
          );
          return Cynthia.send(response);
        } catch (e) {
          console.error(e);
          const response = new CynthiaPluginAPI.ErrorResponse(request.id, "");
          return Cynthia.send(response);
        }
      }
      case "Test": {
        let request: CynthiaPluginAPI.TestRequest = JSON.parse(str);
        // {"id":0,"body":{"as":"OkString","value":"Yes."}}
        let response = new CynthiaPluginAPI.OkStringResponse(
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
