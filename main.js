const CynthiaPluginLoaderVersion = 1;
const devel = process.argv[2] === "--dev" || process.argv[2] === "--short";
const verbose =
	process.argv[2] === "--dev" ||
	process.argv[2] === "--short" ||
	process.argv[2] === "--verbose";
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
const stripAnsiCodes = (str) =>
	str.replace(
		/[\u001b\u009b][[()#;?]*(?:[0-9]{1,4}(?:;[0-9]{0,4})*)?[0-9A-ORZcf-nqry=><]/g,
		""
	);
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
	connsola2(chalkedname, message) {
		const numberofspaces = 20 - stripAnsiCodes(chalkedname).length;
		let spaces = " ".repeat(numberofspaces);
		if (stripAnsiCodes(chalkedname).length > 15) spaces = " ".repeat(5);
		console.log(chalkedname + spaces + message);
	}
	log(errorlevel, name, content) {
		this.logtofile(name, content);
		this.connsola2(`[${name}]`, content);
	}
	warn(content) {
		this.logtofile("WARN", content);
		this.connsola2(`[${chalk.hex("#c25700")("WARN")}]`, content);
	}
	error(content) {
		this.logtofile("ERROR", content);
		this.connsola2(
			`[${chalk.redBright("ERROR")}]`,
			chalk.bgBlack.red(content)
		);
	}
	info(content) {
		this.logtofile("INFO", content);
		this.connsola2(`[${chalk.hex("#6699ff")("INFO")}]`, content);
	}
	silly(content) {
		this.logtofile("SILLY", content);
		this.connsola2(`[${chalk.white("SILLY :3")}]`, chalk.bgBlack.red(content));
	}
	fatal(content) {
		this.logtofile("FATAL", content);
		this.connsola2(`[${chalk.bgBlack.red("FATAL")}]`,
			content
		);
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
if (verbose) lt = new tslog.Logger();
else lt = new logging(logfilename);
const tell = lt;
// Plugin loader
const cynthiabase = {
	modifyOutputHTML: [
		(htmlin) => {
			return htmlin;
		},
	],
	modifyBodyHTML: [
		(htmlin) => {
			return htmlin;
		},
	],
	expressActions: [(expressapp) => {}],
};
fs.readdirSync("./plugins", { withFileTypes: true })
	.filter((dirent) => dirent.isDirectory())
	.map((dirent) => dirent.name)
	.forEach((pluginfolder) => {
		if (pluginfolder.endsWith("-disabled")) return;
		function linklog(displaylinked) {
			displaylinkedfat = chalk.bold(displaylinked);
			tell.log(
				0,
				chalk.reset.hex("5787b8").italic("Plugins"),
				`⬅️➕ Linking ${chalk.dim.magentaBright(
					plugin_package_json.name
				)}.${displaylinkedfat} to ${chalk.dim.yellowBright(
					"cynthiacms"
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
			`⬅️➕ Loading plugin: ${chalk.dim.magentaBright(
				plugin_package_json.name
			)}...`
		);
		const plugin = require(path.join(
			__dirname,
			"plugins/",
			pluginfolder,
			plugin_package_json.main
		));
		if (plugin.CyntiaPluginCompat !== CynthiaPluginLoaderVersion) {
			tell.error(
				`${
					plugin_package_json.name
				}: This plugin was written for a different`
			);
			tell.error(
				`Cynthia Plugin Loader. (Plugin: ${chalk.bold.italic(
					plugin.CyntiaPluginCompat
				)}, Cynthia: ${chalk.bold.italic(CynthiaPluginLoaderVersion)})`
			);
			return;
		}
		if (typeof plugin.modifyOutputHTML === "function") {
			linklog(chalk.greenBright("modifyOutputHTML"));
			cynthiabase.modifyOutputHTML.push(plugin.modifyOutputHTML);
		}
		if (typeof plugin.expressActions === "function") {
			linklog(chalk.blueBright("expressActions"));
			cynthiabase.expressActions.push(plugin.expressActions);
		}
		if (typeof plugin.modifyBodyHTML === "function") {
			linklog(chalk.green("modifyBodyHTML"));
			cynthiabase.modifyBodyHTML.push(plugin.modifyBodyHTML);
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
			tell.log(
				0,
				chalk.reset.cyanBright("Modes"),
				`↘️➕ Loaded mode: '${b[0]}'.`
			);
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
	cynthiabase.modifyBodyHTML.forEach((modifier) => {
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
	cynthiabase.modifyOutputHTML.forEach((modifier) => {
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
		tell.log(0, "GET / 500", `➡️❌		"${req.url}"`);
		res.sendStatus(500);
	} else {
		tell.log(0, "GET / 200", `➡️✔️		"${req.url}"`);
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
cynthiabase.expressActions.forEach((action) => {
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
