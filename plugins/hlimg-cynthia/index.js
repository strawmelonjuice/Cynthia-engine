const path = require("path");
const hlimg_config = require(path.join(__dirname, "/config.json"));
const express = require("express");
module.exports = {
	exportbody (htmlin) {
		return `${htmlin}<script id="hlimg-options" type="application/json">
        ${JSON.stringify(
			hlimg_config
		)}</script>
<script defer type="module" src="/hl-img/hl-img.min.js"></script>`;
	},
    expressactions(expressapp) {
        expressapp.use(
					"/hl-img",
					express.static(path.join(__dirname, "/node_modules/hl-img/dist/"))
				);
    }
};
