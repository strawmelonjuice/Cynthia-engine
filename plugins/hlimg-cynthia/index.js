const path = require("path");
const hlimg_config = require(path.join(__dirname, "/config.json"));
const express = require("express");
module.exports = {
	CyntiaPluginCompat: 1,
	modifyBodyHTML (htmlin) {
		return `${htmlin}<script id="hlimg-options" type="application/json">
        ${JSON.stringify(
			hlimg_config
		)}</script>
<script defer type="module" src="/hl-img/hl-img.min.js"></script>`;
	},
    expressActions(expressapp) {
        expressapp.use(
					"/hl-img",
					express.static(path.join(__dirname, "/node_modules/hl-img/dist/"))
				);
    }
};
