/*
 * Copyright (c) 2024, MLC 'Strawmelonjuice' Bloeiman
 *
 * Licensed under the GNU AFFERO GENERAL PUBLIC LICENSE Version 3, see the LICENSE file for more information.
 */

import {
  type CynthiaPassed,
  CynthiaWebResponderApi,
  type WebRequest,
  IncomingWebRequest,
  type ResponderResponse,
} from "cynthia-plugin-api/main";
export const Plugincompat = 3.2;
export interface PluginBase {
  modifyResponseHTML: Array<
    (htmlin: string, Cynthia: typeof CynthiaPassed) => string
  >;
  modifyResponseHTMLBodyFragment: Array<
    (htmlin: string, Cynthia: typeof CynthiaPassed) => string
  >;
  modifyRequest: Array<
    (req: WebRequest, Cynthia: typeof CynthiaPassed) => void
  >;
  onLoad: Array<(Cynthia: typeof CynthiaPassed) => void>;
  onClearInterval: Array<(Cynthia: typeof CynthiaPassed) => void>;
}
export const newPluginBase: PluginBase = {
  modifyResponseHTML: [
    (htmlin: string, Cynthia: typeof CynthiaPassed) => {
      // Make no changes. Return unchanged.
      return htmlin;
    },
  ],
  modifyResponseHTMLBodyFragment: [
    (htmlin: string, Cynthia: typeof CynthiaPassed) => {
      // Return with a little comment.
      return `${htmlin}\n<!-- test... Body modifier Node plugins enabled! -->\n`;
    },
  ],
  modifyRequest: [
    (req: WebRequest, Cynthia: typeof CynthiaPassed) => {
      // Make no changes. Return unchanged.
      // This function doesn't actually return. It just sends out `Cynthia.answer(() => { return response });` if capturing.
    },
    (req: WebRequest, Cynthia: typeof CynthiaPassed) => {
      req.get("/pltest*", () => {
        const a: ResponderResponse = {
          headers: [],
          body: "This is a test response.",
        };
        return a;
      });
    },
  ],
  onLoad: [
    (Cynthia: typeof CynthiaPassed) => {
      // Do nothing.
    },
  ],
  onClearInterval: [
    (Cynthia: typeof CynthiaPassed) => {
      // Do nothing.
    },
  ],
};
export interface PluginPackageJson {
  "cynthia-plugin": string;
  version: string;
  "cynthia-plugin-compat": number;
}
