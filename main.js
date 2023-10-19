const devel = process.argv[2] === "--dev" || process.argv[2] === "--short";
const express = require("express");
const dotenv = require("dotenv");
const fs = require("fs");
const tar = require("tar");
const path = require("path");
const handlebars = require("handlebars");
const { parse } = require("comment-json");
const tslog = require("tslog");
const connsola = new tslog.Logger();
const MarkdownIt = require("markdown-it");
const chalk = require("chalk");
const { raw } = require("body-parser");
const md = new MarkdownIt({
	html: true,
	linkify: true,
	typographer: true,
});
const Axios = require("axios");
const pjson = require("./package.json");
// Logger
class logging {
	logfile;
	constructor(logfile) {
		this.logfile = logfile;
		this.info(`Logging to "${logfilename}".`);
	}
	logtofile(cat, msg) {
		fs.writeFileSync(
			this.logfile,
			`\n[${cat} ${new Date().toLocaleTimeString()}] ${msg}`,
			{ flag: "a" }
		);
	}
	log(errorlevel, name, content) {
		connsola.log(errorlevel, name, content);
		this.logtofile(name, content);
	}
	warn(content) {
		this.logtofile("WARN", content);
		connsola.warn(content);
	}
	error(content) {
		this.logtofile("ERROR", content);
		connsola.error(content);
	}
	info(content) {
		this.logtofile("INFO", content);
		connsola.info(content);
	}
	silly(content) {
		this.logtofile("SILLY", content);
		connsola.silly(content);
	}
	fatal(content) {
		this.logtofile("FATAL", content);
		connsola.fatal(content);
	}
}
let logfilename;
let starttime;
{
	starttime = new Date(Date.now());
	logfilename = `./logs/log_${starttime.getDate()}-${starttime.getMonth()}-${starttime.getFullYear()}.log`;
}
if (!fs.existsSync("./logs")) {
	fs.mkdirSync("./logs");
}
if (devel) lt = new tslog.Logger();
else lt = new logging(logfilename);
const tell = lt;

// Plugin loader
const cynthiabase = {
	exportoutput: [
		(htmlin) => {
			return htmlin;
		},
	],
	exportbody: [
		(htmlin) => {
			return htmlin;
		},
	],
	expressactions: [(expressapp) => {}],
};
fs.readdirSync("./plugins", { withFileTypes: true })
	.filter((dirent) => dirent.isDirectory())
	.map((dirent) => dirent.name)
	.forEach((pluginfolder) => {
		function linklog(displaylinked) {
			displaylinkedfat = chalk.bold(displaylinked);
			tell.log(
				0,
				chalk.reset.hex("5787b8").italic("Plugins"),
				`Linking ${chalk.dim.magentaBright(
					plugin_package_json.name
				)}.${displaylinkedfat} to ${chalk.dim.yellowBright(
					"cynthiabase"
				)}.${displaylinkedfat}...`
			);
		}
		const plugin_package_json = require(path.join(
			__dirname,
			"plugins/",
			pluginfolder,
			"/package.json"
		));
		tell.log(
			0,
			chalk.reset.hex("5787b8").italic("Plugins"),
			`Loading plugin: ${chalk.dim.magentaBright(plugin_package_json.name)}...`
		);
		const plugin = require(path.join(
			__dirname,
			"plugins/",
			pluginfolder,
			plugin_package_json.main
		));
		if (typeof plugin.exportoutput === "function") {
			linklog(chalk.greenBright("exportoutput"));
			cynthiabase.exportoutput.push(plugin.exportoutput);
		}
		if (typeof plugin.expressactions === "function") {
			linklog(chalk.blueBright("expressactions"));
			cynthiabase.expressactions.push(plugin.expressactions);
		}
		if (typeof plugin.exportbody === "function") {
			linklog(chalk.greenBright("exportbody"));
			cynthiabase.exportbody.push(plugin.exportbody);
		}
	});
function parseBool(bool) {
	if (bool === "true" || bool === "1" || bool === 1 || bool === true)
		return true;
	else return false;
}
if (!fs.existsSync(path.join(__dirname, "./.env")) || devel) {
	tell.warn(
		`${path.join(
			__dirname,
			"./.env"
		)} does not exist. Writing a clean CynthiaConfig.`
	);
	try {
		tar.extract({
			file: path.join(__dirname, "./clean-cyn.tar.gz"),
			cwd: path.join(__dirname),
			sync: true,
		});
	} catch (err) {
		tell.warn("Could not create clean CynthiaConfig. Exiting.");
		tell.error(err);
		process.exit(1);
	}
	tell.warn("Clean CynthiaConfig written! Please adjust then restart Cynthia!");
	if (!devel) process.exit(0);
	else
		tell.warn(
			"Not exiting because Cynthia is in dev mode! Do not make any changes to root CynthiaConfig in dev mode as they will not be recorded."
		);
} else {
	tell.log(
		1,
		"CONFIG",
		`Loading configuration from "${path.join(__dirname, "./.env")}".`
	);
}
dotenv.config();
function HandlebarsAsHTML(file, variables) {
	const template = fs.readFileSync(file).toString();
	// Compile said template
	const compiled = handlebars.compile(template);
	const html = compiled(variables);
	return html;
}
const modes = (() => {
	const d = {};
	fs.readdirSync(path.join(__dirname, "./cynthia_config/modes")).forEach(
		(file) => {
			const b = parse(
				fs.readFileSync(path.join(__dirname, "./cynthia_config/modes", file), {
					encoding: "utf8",
				})
			);
			tell.info(`Loaded mode: '${b[0]}'!`);
			d[b[0]] = b[1];
		}
	);
	return d;
})();
function returnpagemeta(id) {
	let d;
	parse(
		fs.readFileSync(path.join(__dirname, "/site/published.jsonc"), {
			encoding: "utf8",
		})
	).forEach((page) => {
		if (page.id === id) {
			d = page;
		}
	});
	return d;
}
async function ReturnPage(id, currenturl) {
	// Get page meta info
	const pagemeta = returnpagemeta(id);
	// Get info about what template to use
	if (
		pagemeta.mode === undefined ||
		pagemeta.mode == null ||
		pagemeta.mode === ""
	)
		pagemode = "default";
	else pagemode = pagemeta.mode;
	if (pagemeta.type === "post") pagetype = "post";
	else pagetype = "page";
	const handlebarsfile = modes[pagemode].handlebar[pagetype];
	// Get actual page content
	let rawpagecontent;
	switch (pagemeta.content.location) {
		case "inline":
			rawpagecontent = pagemeta.content.raw;
			break;
		case "external":
			rawpagecontent = (await Axios.default.get(pagemeta.content.url).then())
				.data;
			break;
		case "external-direct":
			return (await Axios.default.get(pagemeta.content.url).then()).data;
		case "redirect":
			return { do: "relocation", url: pagemeta.content.url };
		default:
			rawpagecontent = fs.readFileSync(
				path.join(__dirname, "/site/pages/", pagemeta.content.path),
				{
					encoding: "utf8",
				}
			);
			break;
	}
	let pagecontent;
	switch (pagemeta.content.type.toLowerCase()) {
		case "html" || "webfile":
			pagecontent = `<div>${rawpagecontent}</div>`;
			break;
		case "plain" || "text" || "plaintext" || "raw":
			pagecontent = `<div><pre>${rawpagecontent
				.replaceAll("&", "&amp;")
				.replaceAll("<", "&lt;")
				.replaceAll(">", "&gt;")
				.replaceAll('"', "&quot;")
				.replaceAll("'", "&#039;")}</pre></div>`;
			break;
		default:
			pagecontent = `<div>${md.render(rawpagecontent)}</div>`;
			break;
	}
	// Prepare menu links
	let menulinks = "";
	if (
		pagemeta.menulinkoverride === undefined ||
		pagemeta.menulinkoverride == null ||
		pagemeta.menulinkoverride.length === 0
	)
		menu_links = modes[pagemode].menulinks;
	else menu_links = pagemeta.menulinksoverride;
	menu_links.forEach((link) => {
		if (link.href === currenturl)
			menulinks = `${menulinks}<a href="${link.href}" class="active">${link.name}</a>`;
		else menulinks = `${menulinks}<a href="${link.href}">${link.name}</a>`;
	});
	// Load stylesheet and head contents
	stylesheet = fs.readFileSync(
		path.join(__dirname, "/cynthia_config/styles", modes[pagemode].stylefile),
		{
			encoding: "utf8",
		}
	);
	// console.log(stylesheet);
	const headstuff = `<style>
	${stylesheet}
	</style>
	<title>${pagemeta.title} ﹘ ${modes[pagemode].sitename}</title>
	<script>
		const pagemetainfo = JSON.parse(${JSON.stringify(pagemeta)});
	</script>
	`;
	// Run body modifier plugins.
	cynthiabase.exportbody.forEach((modifier) => {
		pagecontent = modifier(pagecontent);
	});

	// Unite the template with it's content and return it to the server
	let page = `<!-- Generated and hosted through Cynthia v${
		pjson.version
	}, by Strawmelonjuice. 
Also see: https://github.com/strawmelonjuice/CynthiaCMS-JS/blob/main/README.MD
-->
	${HandlebarsAsHTML(
		path.join("./cynthia_config/templates/", `${handlebarsfile}.handlebars`),
		{
			head: headstuff,
			content: pagecontent,
			menulinks: menulinks,
		}
	)}`;
	cynthiabase.exportoutput.forEach((modifier) => {
		page = modifier(page);
	});
	// console.log("HTML:" + page);
	return `<!DOCTYPE html>${page}</html>`;
}
async function CynthiaRespond(id, req, res) {
	let anyerrors = true;
	try {
		const cynspon = await ReturnPage(id, req.url);
		if (typeof cynspon !== "object") {
			res.send(cynspon);
			anyerrors = false;
		} else {
			if (cynspon.do === "relocation") {
				res.redirect(302, cynspon.url);
				console.log(`Redirecting '${req.url}' to '${cynspon.url}'.`);
				anyerrors = false;
			}
		}
	} catch {
		anyerrors = true;
	}
	if (anyerrors) {
		tell.log(0, "500", `[GET] ➡️❌   "${req.url}"`);
		res.sendStatus(500);
	} else {
		tell.log(0, "200", `[GET] ➡️✔️   "${req.url}"`);
	}
}
const app = express();
app.get("/", async (req, res) => {
	let pid = "";
	if (typeof req.query.p !== "undefined") pid = req.query.p;
	if (typeof req.query.page !== "undefined") pid = req.query.page;
	if (typeof req.query.post !== "undefined") pid = req.query.post;
	if (typeof req.query.id !== "undefined") pid = req.query.id;
	if (pid !== "") {
		CynthiaRespond(pid, req, res);
	} else {
		CynthiaRespond("root", req, res);
	}
});
cynthiabase.expressactions.forEach((action) => {
	action(app);
});
app.get("/p/:id", async (req, res) => {
	const id = req.params.id;
	CynthiaRespond(id, req, res);
});
app.use("/assets", express.static(path.join(__dirname, "/site/assets/")));
if (process.argv[2] === "--short") {
	tell.info("So far so good! Closing now because Cynthia is in CI mode.");
	process.exit(0);
} else {
	app.listen(process.env.PORT, () => {
		tell.info(`⚡️ Running at http://localhost:${process.env.PORT}/`);
	});
}
