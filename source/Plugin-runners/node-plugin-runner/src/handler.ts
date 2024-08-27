/*
 * Copyright (c) 2024, MLC 'Strawmelonjuice' Bloeiman
 *
 * Licensed under the GNU AFFERO GENERAL PUBLIC LICENSE Version 3, see the LICENSE file for more information.
 */

import * as process from "node:process";
import {
  type ContentRenderRequest,
  Cynthia,
  CynthiaPassed,
  ErrorResponse,
  type GenericRequest,
  type IncomingWebRequest,
  OkStringResponse,
  type PostlistRenderRequest,
  terminalOut as console,
  type TestRequest,
  WebRequest,
} from "cynthia-plugin-api/main";
import * as handlebars from "handlebars";
import * as fs from "node:fs";
import type { PluginBase } from "./types/internal_plugins";

export default async function handle(
  incoming: string,
  cynthiabase: PluginBase,
) {
  if (incoming.startsWith("parse: ")) {
    Cynthia.console.debug("Got a request.");
    Cynthia.console.debug(`Buffer: ${incoming}`);
    const requestAsString = incoming.replace("parse: ", "");
    let request: GenericRequest;
    try {
      request = JSON.parse(requestAsString);
    } catch (_e) {
      Cynthia.console.error("Failed to parse JSON: " + requestAsString);
      const response = new ErrorResponse(
        0,
        "Expected JSON, got: " + requestAsString,
      );
      return Cynthia.send(response);
    }
    switch (request.body.for) {
      case "Exit": {
        console.error("Exiting...");
        return process.exit(0);
      }
      case "WebRequest": {
        const request: IncomingWebRequest = JSON.parse(requestAsString);
        const req: WebRequest = new WebRequest(request.id, {
          method: request.body.method,
          uri: request.body.uri,
          headers: request.body.headers,
        });
        for (const modifier of cynthiabase.modifyRequest) {
          modifier(req, CynthiaPassed);
        }
        return req.escalate();
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
          let htmlBody = compiled(request.body.template_data);
          for (const modifier of cynthiabase.modifyResponseHTMLBodyFragment) {
            htmlBody = modifier(
              htmlBody,
              request.body.template_data.meta,
              CynthiaPassed,
            );
          }
          const response = new OkStringResponse(request.id, htmlBody);
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
          let htmlBody = compiled(request.body.template_data);
          for (const modifier of cynthiabase.modifyResponseHTMLBodyFragment) {
            htmlBody = modifier(
              htmlBody,
              request.body.template_data.meta,
              CynthiaPassed,
            );
          }
          const response = new OkStringResponse(request.id, htmlBody);
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
    console.log(`Got: ${incoming}`);
  }
}
