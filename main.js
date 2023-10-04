const devel = true;
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
const { raw } = require("body-parser");
const md = new MarkdownIt();
const pjson = require('./package.json');
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
			{ flag: "a" },
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
if (devel) lt = new tslog.Logger(); else lt = new logging(logfilename);
const tell = lt;
function parseBool(bool) {
	if (bool === "true" || bool === "1" || bool === 1 || bool === true)
		return true;
	else return false;
}
if ((!fs.existsSync(path.join(__dirname, "./.env"))) || (devel)) {
	tell.warn(
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
		tell.warn("Could not create clean CynthiaConfig. Exiting.");
		tell.error(err);
		process.exit(1);
	}
	tell.warn(
		"Clean CynthiaConfig written! Please adjust then restart Cynthia!",
	);
	if (!devel) process.exit(0); else tell.warn("Not exiting because Cynthia is in dev mode! Do not make any changes to root CynthiaConfig in dev mode as they will not be recorded.")
} else {
	tell.log(
		1,
		"CONFIG",
		`Loading configuration from "${path.join(__dirname, "./.env")}".`,
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
	fs.readdirSync(path.join(__dirname, "./_cynthia/config/modes")).forEach(
		(file) => {
			const b = parse(
				fs.readFileSync(path.join(__dirname, "./_cynthia/config/modes", file), {
					encoding: "utf8",
				}),
			);
			tell.info(`Loaded mode: '${b[0]}'!`);
			d[b[0]] = b[1];
		},
	);
	return d;
})();
function returnpagemeta(id) {
	let d;
	(parse(
		fs.readFileSync(path.join(__dirname, "./_cynthia/cynthiameta.jsonc"), {
			encoding: "utf8",
		}),
	)).forEach((page) => {
		if (page.id === id) {
			d = page;
		}
	});
	return d;
}
function ReturnPage(id, currenturl) {
	// Get page meta info
	const pagemeta = returnpagemeta(id);
	// Get info about what template to use
	if (pagemeta.mode === undefined || pagemeta.mode == null || pagemeta.mode === "") pagemode = 'default'; else pagemode = pagemeta.mode;
	if (pagemeta.type === "post") pagetype = "post"; else pagetype = "page";
	const handlebarsfile = modes[pagemode].handlebar[pagetype];
	// Get actual page content
	let rawpagecontent;
	switch (pagemeta.content.location) {
		case "inline":
			rawpagecontent = pagemeta.content.raw;
			break;

		default:
			rawpagecontent = fs.readFileSync(path.join(__dirname, "./pages/", pagemeta.content.path), {
				encoding: "utf8",
			})
			break;
	}
	let pagecontent;
	switch ((pagemeta.content.type).toLowerCase()) {
		case "html" || "webfile":
			tell.silly("Serving html");
			pagecontent = `<div>${rawpagecontent}</div>`;
			break;
		case "plain" || "text" || "plaintext" || "raw":
			tell.silly("Serving plaintext");
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
	console.log(`Current url: ${currenturl}`)
	if (
		pagemeta.menulinkoverride === undefined ||
		pagemeta.menulinkoverride == null ||
		pagemeta.menulinkoverride.length === 0
	)
		menu_links = modes[pagemode].menulinks;
	else menu_links = pagemeta.menulinksoverride;
	menu_links.forEach((link) => {
		if (link.href === currenturl) menulinks = `${menulinks}<a href="${link.href}" class="active">${link.name}</a>`; else menulinks = `${menulinks}<a href="${link.href}">${link.name}</a>`;
	})
	// Load stylesheet and head contents
	stylesheet = fs.readFileSync(path.join(__dirname, "/_cynthia/files/styles", modes[pagemode].stylefile), {
		encoding: "utf8",
	});
	// console.log(stylesheet);
	const headstuff = 
	`<style>
	${stylesheet}
	</style>
	<title>${pagemeta.title} ﹘ ${modes[pagemode].sitename}</title>
	<script>
		const pagemetainfo = JSON.parse(${JSON.stringify(pagemeta)});
	</script>
	`;
	// Unite the template with it's content and return it to the server
	page = 
	`<!-- Generated and hosted through Cynthia v${pjson.version}, by Strawmelonjuice. 
Also see: https://github.com/strawmelonjuice/CynthiaCMS-JS/blob/main/README.MD
-->
	${HandlebarsAsHTML(
		path.join("./_cynthia/files/templates/", `${handlebarsfile}.handlebars`),
		{
			head: headstuff,
			content: pagecontent,
			menulinks: menulinks
		}
	)}`;
	// console.log(page);
	return page;
}

const app = express();
app.get("/", async (req, res) => {
	let anyerrors = false;
	try {
		res.send(ReturnPage("root", "/"));
		anyerrors = false;
	} catch {
		anyerrors = true;
	}
	if (anyerrors) {
		tell.warn(`[GET] ➡️❌   "${req.url}"`);
	} else {
		tell.log(0, "OK", `[GET] ➡️✔️   "${req.url}"`);
	}
});

app.get('/p/:id', async (req, res) => {
	let anyerrors = false;
	const id = req.params.id;
	try {
		res.send(ReturnPage(id, `/p/${id}`));
		anyerrors = false;
	} catch {
		anyerrors = true;
	}
	if (anyerrors) {
		tell.warn(`[GET] ➡️❌   "${req.url}"`);
	} else {
		tell.log(0, "OK", `[GET] ➡️✔️   "${req.url}"`);
	}
});
app.use("/assets", express.static(path.join(__dirname, "/assets/")));
app.use("/hl-img", express.static(path.join(__dirname, "/node_modules/hl-img/dist/")));
app.listen(process.env.PORT, () => {
	tell.info(`⚡️ Running at http://localhost:${process.env.PORT}/`);
});
