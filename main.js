const express = require("express");
const dotenv = require("dotenv");
const fs = require("fs");
const tar = require("tar");
const path = require("path");
const handlebars = require("handlebars");
const { parse } = require("comment-json");
function parseBool(bool) {
	if (bool === "true" || bool === "1" || bool === 1 || bool === true)
		return true;
	else return false;
}
if (!fs.existsSync(path.join(__dirname, "./.env"))) {
	console.warn(
		`${path.join(
			__dirname,
			"./.env",
		)} does not exist. Writing a clean CynthiaConfig.`,
	);
	try {
         tar.extract({
                file: path.join(__dirname, "./clean-cyn.tar.gz"),
                cwd: path.join(__dirname),
                sync: true,
            });
	} catch (err) {
		console.warn("Could not create clean CynthiaConfig. Exiting.");
		console.error(err);
		process.exit(1);
	}
	console.warn("Clean CynthiaConfig written! Please adjust then restart Cynthia!");
	process.exit(0);
} else {
	console.log(
		1,
		"CONFIG",
		`Loading configuration from "${path.join(
			__dirname,
			"./.env",
		)}".`,
	);
}
dotenv.config();
const ahtime = new Date(Date.now());
function HandlebarsAsHTML(file, variables) {
	const template = fs.readFileSync(file).toString();
	// Compile said template
	const compiled = handlebars.compile(template);
	const html = compiled(variables);
	return html;
}
const modes = (() => {
let d;
fs.readdirSync(path.join(__dirname, "./_cynthia/config/modes")).forEach(file => {
    d[file] = parse(readFileSync(path.join(__dirname, "./_cynthia/config/modes", file), {encoding: "utf8"}));
})
return d;
})()
console.log(modes);
function LoadPage(id) {
		
}

const app = express();
	app.get("/*", (req, res) => {
		let anyerrors;
		switch (req.url) {
			case "/":
				LoadPage("root");
				// res.send(preloadedresponses.html.index);
				anyerrors = false;
				break;

			default:
				res.status(404);
				res.send("Not sure what you need.");
				anyerrors = true;
				break;
		}
		if (anyerrors) {
			tell.warn(`[GET] ➡️❌   "${req.url}"`);
		} else {
			tell.log(0, "OK", `[GET] ➡️✔️   "${req.url}"`);
		}
	});
app.listen(process.env["PORT"], () => {
	console.info(`⚡️ Running at http://localhost:${process.env["PORT"]}/`);
});
