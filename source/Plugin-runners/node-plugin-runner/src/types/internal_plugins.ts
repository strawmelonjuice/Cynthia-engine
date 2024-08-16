/*
 * Copyright (c) 2024, MLC 'Strawmelonjuice' Bloeiman
 *
 * Licensed under the GNU AFFERO GENERAL PUBLIC LICENSE Version 3, see the LICENSE file for more information.
 */

import type {
  CynthiaPassed,
  CynthiaWebResponderApi,
  IncomingWebRequest,
} from "cynthia-plugin-api/main";
export const Plugincompat = 3.2;
export interface PluginBase {
  modifyResponseHTML: [
    (htmlin: string, Cynthia: typeof CynthiaPassed) => string,
  ];
  modifyResponseHTMLBodyFragment: [
    (htmlin: string, Cynthia: typeof CynthiaPassed) => string,
  ];
  modifyRequest: [
    (
      WebRequest: IncomingWebRequest,
      Cynthia: typeof CynthiaWebResponderApi,
    ) => void,
  ];
  onLoad: [() => void];
  onClearInterval: [() => void];
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
    (
      WebRequest: IncomingWebRequest,
      Cynthia: typeof CynthiaWebResponderApi,
    ) => {
      // Make no changes. Return unchanged.
      // This function doesn't actually return. It just sends out `Cynthia.answer(() => { return response });` if capturing.
    },
  ],
  onLoad: [
    () => {
      // Do nothing.
    },
  ],
  onClearInterval: [
    () => {
      // Do nothing.
    },
  ],
};
export interface PluginPackageJson {
  "cynthia-plugin": string;
  version: string;
  "cynthia-plugin-compat": number;
}
