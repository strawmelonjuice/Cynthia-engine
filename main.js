const express = require("express");
const dotenv = require("dotenv");
const fs = require("fs");
const tar = require("tar");
const path = require("path");
const handlebars = require("handlebars");
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
// const preloadedresponses = {
// 	html: {
// 		index: HandlebarsAsHTML("./assets/hb/index.handlebars", vars),
// 	},
// };
const app = express();
	// const pino = require("pino-http")();
	// app.use(pino);
	app.use(express.json());
	app.use(require("body-parser").urlencoded({ extended: false }));
	app.use(
		session({
			secret: daSecret,
			resave: false,
			saveUninitialized: true,
			cookie: { secure: "auto" },
		}),
	);

	app.use("/assets", express.static(path.join(__dirname, "./assets/")));

	app.get("/*", (req, res) => {
		let anyerrors;
		switch (req.url) {
			case "/":
				res.send(HandlebarsAsHTML("./assets/hb/index.handlebars", vars));
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
app.listen(vars.port, () => {
	console.info(`⚡️ Running at http://localhost:${vars.port}/`);
	console.log(
		0,
		"PHP",
		`PHP parser as a child process on http://${PHPParserHTTP}`,
	);
});
