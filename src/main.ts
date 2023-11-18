const CynthiaPluginLoaderVersion = 1;
const devel =
  process.argv[2] === "--dev" ||
  process.argv[2] === "--short" ||
  process.argv[3] === "--dev" ||
  process.argv[3] === "--short";
if (devel) console.log("Development mode is on.");
const verbose =
  process.argv[2] === "--verbose" ||
  process.argv[2] === "-v" ||
  process.argv[2] === "--loud" ||
  process.argv[3] === "--verbose" ||
  process.argv[3] === "-v" ||
  process.argv[3] === "--loud";
// --loud was added because nodemon kept picking --verbose up.
if (verbose) console.log("Verbose mode is on.");
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
// doesn't work in ts: const pjson = require("./package.json");
// so:
const pjsonstring = fs.readFileSync(path.join(__dirname, "../package.json"), {
	encoding: "utf8",
	flag: "r",
});
const pjson = JSON.parse(pjsonstring);
const stripAnsiCodes = (str) =>
	str.replace(
		// biome-ignore lint/suspicious/noControlCharactersInRegex:
		/[\u001b\u009b][[()#;?]*(?:[0-9]{1,4}(?:;[0-9]{0,4})*)?[0-9A-ORZcf-nqry=><]/g,
		"",
	);
// Pre plugin loader
const cynthiabase = {
	modifyOutputHTML: [
		(htmlin: string) => {
			// Make no changes. Return unchanged.
			return htmlin;
		},
	],
	modifyBodyHTML: [
		(htmlin: String) => {
			// Make no changes. Return unchanged.
			return htmlin;
		},
	],
	expressActions: [(expressapp: typeof express) => {}],
	LogReader: [(type: string, msg: string) => {}],
};
// Logger
class logging {
	logfile: string;
	constructor(logfile: string) {
		this.logfile = logfile;
		this.info(`🖊 Logging to "${logfilename}".`);
	}
	logtofile(cat: string, msg: string) {
		fs.writeFileSync(
			this.logfile,
			`\n[${cat} ${new Date().toLocaleTimeString()}] ${msg}`,
			{ flag: "a" },
		);
	}
	connsola2(chalkedname: string, message: string) {
		const numberofspaces = 20 - stripAnsiCodes(chalkedname).length;
		let spaces = " ".repeat(numberofspaces);
		if (stripAnsiCodes(chalkedname).length > 15) spaces = " ".repeat(5);
		console.log(chalkedname + spaces + message);
		cynthiabase.LogReader.forEach((action) => {
			action(stripAnsiCodes(chalkedname), message);
		});
	}
	log(_errorlevel: number, name:string, content:string) {
		this.logtofile(name, content);
		this.connsola2(`[${name}]`, content);
	}
	warn(content: string) {
		this.logtofile("WARN", content);
		this.connsola2(`[${chalk.hex("#c25700")("WARN")}]`, content);
	}
	error(content: string) {
		this.logtofile("ERROR", content);
		this.connsola2(`[${chalk.redBright("ERROR")}]`, chalk.bgBlack.red(content));
	}
	info(content: string) {
		this.logtofile("INFO", content);
		this.connsola2(`[${chalk.hex("#6699ff")("INFO")}]`, content);
	}
	silly(content: string) {
		this.logtofile("SILLY", content);
		this.connsola2(`[${chalk.white("SILLY :3")}]`, chalk.bgBlack.red(content));
	}
	fatal(content:string) {
		this.logtofile("FATAL", content);
		this.connsola2(`[${chalk.bgBlack.red("FATAL")}]`, content);
	}
}

let logfilename: string;
let starttime: Date;
{
	starttime = new Date(Date.now());
	logfilename = `./logs/log_${starttime.getDate()}-${starttime.getMonth()}-${starttime.getFullYear()}.log`;
}
if (!fs.existsSync("./logs")) {
	fs.mkdirSync("./logs");
}
let lt: logging;
if (verbose) lt = new tslog.Logger();
else lt = new logging(logfilename);
const tell = lt;

let debuglog = (a: string) => {void(a)};
if (verbose)
	debuglog = (a: string) => {
		tell.log(0, "DEBUG:", a);
	};// Plugin loader
fs.readdirSync("./plugins", { withFileTypes: true })
	.filter((dirent) => dirent.isDirectory())
	.map((dirent) => dirent.name)
	.forEach((pluginfolder: string) => {
		if (pluginfolder.endsWith("-disabled")) return;
		function linklog(displaylinked) {
			const displaylinkedfat = chalk.bold(displaylinked);
			tell.log(
				0,
				chalk.reset.hex("5787b8").italic("Plugins"),
				`🧩 Linking ${chalk.dim.magentaBright(
					plugin_package_json.name,
				)}.${displaylinkedfat} to ${chalk.dim.yellowBright(
					"cynthiacms",
				)}.${displaylinkedfat}...`,
			);
		}
		const plugin_package_json = require(path.join(
			__dirname,
			"/../",
			"plugins/",
			pluginfolder,
			"/package.json",
		));
		tell.log(
			0,
			chalk.reset.hex("5787b8").italic("Plugins"),
			`🧩 Loading plugin: ${chalk.dim.magentaBright(
				plugin_package_json.name,
			)}...`,
		);
		const plugin = require(path.join(
			__dirname,
			"/../",
			"plugins/",
			pluginfolder,
			plugin_package_json.main,
		));
		if (plugin.CyntiaPluginCompat !== CynthiaPluginLoaderVersion) {
			tell.error(
				`${plugin_package_json.name}: This plugin was written for a different`,
			);
			tell.error(
				`Cynthia Plugin Loader. (Plugin: ${chalk.bold.italic(
					plugin.CyntiaPluginCompat,
				)}, Cynthia: ${chalk.bold.italic(CynthiaPluginLoaderVersion)})`,
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
		if (typeof plugin.LogReader === "function") {
			linklog(chalk.yellowBright("LogReader"));
			cynthiabase.LogReader.push(plugin.LogReader);
		}
	});
function parseBool(bool: string | number | boolean) {
	return bool === "true" || bool === "1" || bool === 1 || bool === true;
}
if (!fs.existsSync(path.join(__dirname, "/../", "./.env")) || devel) {
	tell.warn(
		`${path.join(
			__dirname,
			"/../",
			"./.env",
		)} does not exist. Writing a clean CynthiaConfig.`,
	);
	try {
		tar.extract({
			file: path.join(__dirname, "/../", "./clean-cyn.tar.gz"),
			cwd: path.join(__dirname, "/../"),
			sync: true,
			keep: false
		});
	} catch (err) {
		tell.warn("Could not create clean CynthiaConfig. Exiting.");
		process.exit(1);
	}
	tell.warn("Clean CynthiaConfig written! Please adjust then restart Cynthia!");
	if (!devel) process.exit(0);
	else
		tell.warn(
			"Not exiting because Cynthia is in dev mode! Do not make any changes to root CynthiaConfig in dev mode as they will not be recorded.",
		);
} else {
	tell.log(
		1,
		"CONFIG",
		`🤔 Loading configuration from "${path.join(
			__dirname,
			"/../",
			"./.env",
		)}".`,
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
	fs.readdirSync(
		path.join(__dirname, "/../", "./cynthia_config/modes"),
	).forEach((file) => {
		const b = parse(
			fs.readFileSync(
				path.join(__dirname, "/../", "./cynthia_config/modes", file),
				{
					encoding: "utf8",
				},
			),
		);
		tell.log(0, chalk.reset.cyanBright("Modes"), `💡 Loaded mode: '${b[0]}'.`);
		d[b[0]] = b[1];
	});
	return d;
})();
function returnpagemeta(id) {
	let d;
	parse(
		fs.readFileSync(path.join(__dirname, "/../", "/site/published.jsonc"), {
			encoding: "utf8",
		}),
	).forEach((page) => {
		if (page.id === id) {
			d = page;
		}
	});
	return d;
}

function ReturnpostlistPage(postlistmetainfo: { filters: {} | undefined; }) {
	if (!(postlistmetainfo.filters == undefined || postlistmetainfo.filters == null)) {
		return ("Filtered page list.");
	} else {
		return ("Unfiltered page list.");
	}
}
async function ReturnPage(id, currenturl) {
	// Get page meta info
	const pagemeta = returnpagemeta(id);
	let pagemode: string;
	// Get info about what template to use
	if (
		pagemeta.mode === undefined ||
		pagemeta.mode == null ||
		pagemeta.mode === ""
	)
		pagemode = "default";
	else pagemode = pagemeta.mode;
	let pagetype: string;
	if (pagemeta.type === "post") pagetype = "post";
	else pagetype = "page";
	const handlebarsfile = modes[pagemode].handlebar[pagetype];
	// Get actual page content
	let rawpagecontent;
	if (pagemeta.postlist != undefined) {
		rawpagecontent = ReturnpostlistPage(pagemeta.postlist);
	} else
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
				path.join(__dirname, "/../", "/site/pages/", pagemeta.content.path),
				{
					encoding: "utf8",
				},
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
	// debuglog("Menu links 1");
	let menu1links = "";
	let menu1_links;
	if (
		pagemeta.menu1linkoverride === undefined ||
		pagemeta.menu1linkoverride == null ||
		pagemeta.menu1linkoverride.length === 0
	)
		menu1_links = modes[pagemode].menu1links;
	else menu1_links = pagemeta.menu1linksoverride;
	if (menu1_links === undefined || menu1_links == null)
		menu1_links = modes[pagemode].menulinks;
	if (menu1_links !== undefined && menu1_links !== null) {
		menu1_links.forEach((link) => {
			if (link.href === currenturl)
				menu1links = `${menu1links}<a href="${link.href}" class="active">${link.name}</a>`;
			else menu1links = `${menu1links}<a href="${link.href}">${link.name}</a>`;
		});
	}
	// debuglog("Menu links 2");
	let menu2links = "";
	let menu2_links;
	if (
		pagemeta.menu2linkoverride === undefined ||
		pagemeta.menu2linkoverride == null ||
		pagemeta.menu2linkoverride.length === 0
	)
		menu2_links = modes[pagemode].menu2links;
	else menu2_links = pagemeta.menu2linksoverride;
	if (menu2_links !== undefined && menu2_links !== null) {
		menu2_links.forEach((link) => {
			if (link.href === currenturl)
				menu2links = `${menu2links}<a href="${link.href}" class="active">${link.name}</a>`;
			else menu2links = `${menu2links}<a href="${link.href}">${link.name}</a>`;
		});
	}
	// Load stylesheet and head contents
	// debuglog("Head construction");

	const stylesheet = fs.readFileSync(
		path.join(
			__dirname,
			"/../",
			"/cynthia_config/styles",
			modes[pagemode].stylefile,
		),
		{
			encoding: "utf8",
		},
	);
	// console.log(stylesheet);
	const headstuff = `<style>
	${stylesheet}
	</style>
	<script src="/jquery/jquery.min.js"></script>
	<title>${pagemeta.title} ﹘ ${modes[pagemode].sitename}</title>
	<script>
		const pagemetainfo = JSON.parse(\`${JSON.stringify(pagemeta)}\`);
	</script>
	`;
	// Run body modifier plugins.
	cynthiabase.modifyBodyHTML.forEach((modifier) => {
		// debuglog(`Body modifier: ${modifier}`);
		pagecontent = modifier(pagecontent);
	});

	// debuglog("Body is ready, going to unitor now.");

	const pageinfoshow = `
	<span class="pageinfosidebar" id="pageinfosidebartoggle" style="transition: all 1s ease-out 0s; width: 0px; font-size: 3em; bottom: 215px; display: none; text-align: right; padding: 0px; cursor: pointer;" onclick="pageinfosidebar_rollout()">➧</span>
	<div class="pageinfosidebar" id="cynthiapageinfoshowdummyelem"></div>`;

	// Unite the template with it's content and return it to the server
	let page = `<!-- Generated and hosted through Cynthia v${pjson.version}, by Strawmelonjuice. 
Also see: https://github.com/strawmelonjuice/CynthiaCMS-JS/blob/main/README.MD
-->
	${HandlebarsAsHTML(
		path.join("./cynthia_config/templates/", `${handlebarsfile}.handlebars`),
		{
			head: headstuff,
			content: pagecontent,
			menu1: menu1links,
			menu2: menu2links,
			infoshow: pageinfoshow
		},
	)}`;
	cynthiabase.modifyOutputHTML.forEach((modifier) => {
		page = modifier(page);
	});
	// console.log("HTML:" + page);

	return `<!DOCTYPE html>${page}<script>${fs.readFileSync(path.join(__dirname, "../src/client.js"), {
	encoding: "utf8",
	flag: "r",
})}</script></html>`;
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
		tell.log(0, "GET / 500", `❎: "${req.url}"`);
		res.sendStatus(500);
	} else {
		tell.log(0, "GET / 200", `✅: "${req.url}"`);
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
app.get("/p/*", async (req, res) => {
	const id = (req.originalUrl.replace("/p/", "")).replace(/\/$/, "");
	if (id == "") {
		res.redirect("/");
		return;
	}
	CynthiaRespond(id, req, res);
});
app.use(
	"/assets",
	express.static(path.join(__dirname, "/../", "/site/assets/")),
);
app.use(
  "/jquery",
  express.static(
    path.join(
      __dirname,
      "/../",
      "node_modules/jquery/dist/"
    )
  )
);
if (process.argv[2] === "--short") {
	tell.info("So far so good! Closing now because Cynthia is in CI mode.");
	process.exit(0);
} else {
	app.listen(process.env.PORT, () => {
		tell.info(`🆙 Running at http://localhost:${process.env.PORT}/`);
	});
}
