/*
 * Copyright (c) 2024, MLC 'Strawmelonjuice' Bloeiman
 *
 * Licensed under the GNU AFFERO GENERAL PUBLIC LICENSE Version 3, see the LICENSE file for more information.
 */

import * as CynthiaPluginAPI from "cynthia-plugin-api/main";
import {Cynthia} from "cynthia-plugin-api/main";
import {terminalOut as console} from "../../node-plugin-api/main";
console.info("Node plugin server starting...");
const cynthiabase = {
	modifyOutputHTML: [
		(htmlin: string, Cynthia: typeof CynthiaPluginAPI.Cynthia) => {
			// Make no changes. Return unchanged.
			return htmlin;
		},
	],
	modifyBodyHTML: [
		(htmlin: string, Cynthia: typeof CynthiaPluginAPI.Cynthia) => {
			// Make no changes. Return unchanged.
			return htmlin;
		},
	],
	requestOptions: [
		(WebRequest: CynthiaPluginAPI.Incoming.WebRequest, Cynthia: typeof CynthiaPluginAPI.Cynthia) => {
			// Make no changes. Return unchanged.
		},
	],
};
process.stdin.resume();
process.stdin.on("data", handle);

function handle(buffer: Buffer) {
	if (buffer.toString().startsWith("parse: ")) {
		const str = buffer.toString().replace("parse: ", "");
		let request: CynthiaPluginAPI.GenericRequest = JSON.parse(str);
		switch (request.body.for) {
			case "Exit": {
				console.error("Exiting...");
				return (process.exit(0));
			}
			case "Test": {
				let request: CynthiaPluginAPI.TestRequest = JSON.parse(str);
				// {"id":0,"body":{"as":"OkString","value":"Yes."}}
				let response = new CynthiaPluginAPI.OkStringResponse(request.id, `Successfully received test request. Test passed with echo: "${request.body.test}"`);
				return Cynthia.send(response);
			}
		}
	} else {
		console.log(`Got: ${buffer}`);
	}
}
