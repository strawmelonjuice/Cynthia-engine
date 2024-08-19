/*
 * Copyright (c) 2024, MLC 'Strawmelonjuice' Bloeiman
 *
 * Licensed under the GNU AFFERO GENERAL PUBLIC LICENSE Version 3, see the LICENSE file for more information.
 */

import {
  type WebRequest,
  IncomingWebRequest,
  type ResponderResponse,
} from "cynthia-plugin-api/main";
import type {
  ContentMetaDataType,
  CynthiaApiPoints,
} from "../../../node-plugin-api/main";
export const Plugincompat = 3.2;
export interface PluginBase {
  modifyResponseHTML: Array<
    (
      htmlin: string,
      metadata: ContentMetaDataType,
      Cynthia: CynthiaApiPoints,
    ) => string
  >;
  modifyResponseHTMLBodyFragment: Array<
    (
      htmlin: string,
      metadata: ContentMetaDataType,
      Cynthia: CynthiaApiPoints,
    ) => string
  >;
  modifyRequest: Array<(req: WebRequest, Cynthia: CynthiaApiPoints) => void>;
  onLoad: Array<(Cynthia: CynthiaApiPoints) => void>;
  onClearInterval: Array<(Cynthia: CynthiaApiPoints) => void>;
}
export const newPluginBase: PluginBase = {
  modifyResponseHTML: [
    (htmlin: string, _1, _2) => {
      // Make no changes. Return unchanged.
      return htmlin;
    },
  ],
  modifyResponseHTMLBodyFragment: [
    (htmlin: string, _1, _2) => {
      // Return with a little comment.
      return `${htmlin}\n<!-- test... Body modifier Node plugins enabled! -->\n`;
    },
  ],
  modifyRequest: [
    (req: WebRequest, Cynthia: CynthiaApiPoints) => {
      // Make no changes. Return unchanged.
      // This function doesn't actually return. It just sends out `Cynthia.answer(() => { return response });` if capturing.
    },
    (req: WebRequest, Cynthia: CynthiaApiPoints) => {
      req.get("/pltest*", () => {
        return "This is a test response.";
      });
    },
  ],
  onLoad: [
    (Cynthia: CynthiaApiPoints) => {
      // Do nothing.
    },
  ],
  onClearInterval: [
    (Cynthia: CynthiaApiPoints) => {
      // Do nothing.
    },
  ],
};
export interface PluginPackageJson {
  "cynthia-plugin": string;
  version: string;
  "cynthia-plugin-compat": number;
}
