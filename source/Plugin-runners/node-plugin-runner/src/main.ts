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

console.info("Config loaded.");
console.debug(`Config: ${JSON.stringify(config)}`);

import {
	terminalOut as console,
	CynthiaPassed,
} from "../../node-plugin-api/main";
import * as handlebars from "handlebars";
import * as fs from "node:fs";
import path from "node:path";

console.info(
	`External Javascript Runtime Server starting in: ${process.argv0}`
);
const cynthiabase = {
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
			Cynthia: typeof CynthiaPluginAPI.CynthiaWebResponderApi
		) => {
			// Make no changes. Return unchanged.
			// This function doesn't actually return. It just sends out `Cynthia.answer(() => { return response });` if capturing.
		},
	],
};

fs.readdirSync("./plugins", { withFileTypes: true })
	.filter((dirent) => dirent.isDirectory())
	.map((dirent) => dirent.name)
	.forEach((pluginfolder: string) => {
		if (pluginfolder.endsWith("-disabled")) return;
		function linklog(displaylinked) {}
		const plugin_package_json = require(path.join(
			__dirname,
			"/../",
			"plugins/",
			pluginfolder,
			"/package.json"
		));

		const plugin = require(path.join(
			__dirname,
			"/../",
			"plugins/",
			pluginfolder,
			plugin_package_json.main
		));
		// if (plugin.CyntiaPluginCompat !== CynthiaPluginLoaderVersion) {
		//   return;
		// }
		if (typeof plugin.modifyOutputHTML === "function") {
			cynthiabase.modifyOutputHTML.push(plugin.modifyOutputHTML);
		}
		if (typeof plugin.requestOptions === "function") {
			cynthiabase.requestOptions.push(plugin.expressActions);
		}
		if (typeof plugin.modifyBodyHTML === "function") {
			cynthiabase.modifyBodyHTML.push(plugin.modifyBodyHTML);
		}
	});

process.stdin.resume();
process.stdin.on("data", handle);

async function handle(buffer: Buffer) {
	if (buffer.toString().startsWith("parse: ")) {
		console.debug("Got a request.");
		console.debug(`Buffer: ${buffer}`);
		const requestAsString = buffer.toString().replace("parse: ", "");
		let request: CynthiaPluginAPI.GenericRequest = JSON.parse(requestAsString);
		switch (request.body.for) {
			case "Exit": {
				console.error("Exiting...");
				return process.exit(0);
			}
			case "WebRequest": {
				console.debug("Got a WebRequest.");
				console.debug(`Request: ${requestAsString}`);
				// Until this is implemented, we will just return EmptyOk.
				let response = new CynthiaPluginAPI.EmptyOKResponse(request.id);
				return Cynthia.send(response);
			}
			case "PostlistRenderRequest": {
				try {
					// streq helper
					// This helper checks if two strings are equal.
					// Usage: {{#if (streq postid "sasfs")}} ... {{/if}}
					handlebars.registerHelper("streq", function (a: string, b: string) {
						return a === b;
					});

					let request: CynthiaPluginAPI.PostlistRenderRequest =
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
						page
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
					handlebars.registerHelper("streq", function (a: string, b: string) {
						return a === b;
					});

					let request: CynthiaPluginAPI.ContentRenderRequest =
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
						page
					);
					return Cynthia.send(response);
				} catch (e) {
					console.error(e);
					const response = new CynthiaPluginAPI.ErrorResponse(request.id, "");
					return Cynthia.send(response);
				}
			}
			case "Test": {
				let request: CynthiaPluginAPI.TestRequest = JSON.parse(requestAsString);
				// {"id":0,"body":{"as":"OkString","value":"Yes."}}
				let response = new CynthiaPluginAPI.OkStringResponse(
					request.id,
					`Successfully received test request. Test passed with echo: "${request.body.test}"`
				);
				return Cynthia.send(response);
			}
		}
	} else {
		console.log(`Got: ${buffer}`);
	}
}
