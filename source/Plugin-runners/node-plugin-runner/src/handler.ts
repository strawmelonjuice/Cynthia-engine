/*
 * Copyright (c) 2024, MLC 'Strawmelonjuice' Bloeiman
 *
 * Licensed under the GNU AFFERO GENERAL PUBLIC LICENSE Version 3, see the LICENSE file for more information.
 */

import * as process from "node:process";
import {
  terminalOut as console,
  type ContentRenderRequest,
  Cynthia,
  CynthiaPassed,
  EmptyOKResponse,
  ErrorResponse,
  type GenericRequest,
  OkStringResponse,
  type PostlistRenderRequest,
  type TestRequest,
} from "cynthia-plugin-api/main";
import * as handlebars from "handlebars";
import * as fs from "node:fs";
import type { PluginBase } from "./types/internal_plugins";

export default async function handle(buffer: Buffer, cynthiabase: PluginBase) {
  if (buffer.toString().startsWith("parse: ")) {
    console.debug("Got a request.");
    console.debug(`Buffer: ${buffer}`);
    const requestAsString = buffer.toString().replace("parse: ", "");
    const request: GenericRequest = JSON.parse(requestAsString);
    switch (request.body.for) {
      case "Exit": {
        console.error("Exiting...");
        return process.exit(0);
      }
      case "WebRequest": {
        console.debug("Got a WebRequest.");
        console.debug(`Request: ${requestAsString}`);
        // Until this is implemented, we will just return EmptyOk.
        const response = new EmptyOKResponse(request.id);
        return Cynthia.send(response);
      }
      case "PostlistRenderRequest": {
        try {
          // streq helper
          // This helper checks if two strings are equal.
          // Usage: {{#if (streq postid "sasfs")}} ... {{/if}}
          handlebars.registerHelper("streq", (a: string, b: string) => a === b);

          const request: PostlistRenderRequest = JSON.parse(requestAsString);
          const template = fs.readFileSync(request.body.template_path, "utf8");
          const compiled = handlebars.compile(template);
          const htmlBody = compiled(request.body.template_data);
          let body = htmlBody;
          for (const modifier of cynthiabase.modifyResponseHTMLBodyFragment) {
            body = modifier(body, CynthiaPassed);
          }
          const response = new OkStringResponse(request.id, body);
          return Cynthia.send(response);
        } catch (e) {
          console.error(e);
          const response = new ErrorResponse(request.id, "");
          return Cynthia.send(response);
        }
      }
      case "ContentRenderRequest": {
        try {
          // streq helper
          // This helper checks if two strings are equal.
          // Usage: {{#if (streq postid "sasfs")}} ... {{/if}}
          handlebars.registerHelper("streq", (a: string, b: string) => a === b);

          const request: ContentRenderRequest = JSON.parse(requestAsString);
          const template = fs.readFileSync(request.body.template_path, "utf8");
          const compiled = handlebars.compile(template);
          const htmlBody = compiled(request.body.template_data);
          let body = htmlBody;
          for (const modifier of cynthiabase.modifyResponseHTMLBodyFragment) {
            body = modifier(body, CynthiaPassed);
          }
          const response = new OkStringResponse(request.id, body);
          return Cynthia.send(response);
        } catch (e) {
          console.error(e);
          const response = new ErrorResponse(request.id, "");
          return Cynthia.send(response);
        }
      }
      case "Test": {
        const request: TestRequest = JSON.parse(requestAsString);
        // {"id":0,"body":{"as":"OkString","value":"Yes."}}
        const response = new OkStringResponse(
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
