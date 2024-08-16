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
  CyntiaPluginCompat: 3.2;
  modifyOutputHTML: [(htmlin: string, Cynthia: typeof CynthiaPassed) => string];
  modifyBodyHTML: [(htmlin: string, Cynthia: typeof CynthiaPassed) => string];
  requestOptions: [
    (
      WebRequest: IncomingWebRequest,
      Cynthia: typeof CynthiaWebResponderApi,
    ) => void,
  ];
}
export interface PluginPackageJson {
  "cynthia-plugin": string;
  "cynthia-plugin-version": string;
  "cynthia-plugin-compat": number;
}
