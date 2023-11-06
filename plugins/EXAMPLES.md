# Example of a `package.json`.

```json
{
  "name": "@my-cynthia-plugins/amazing-plugin",
  "version": "1.0.1",
  "description": "An amazing example plugin for CynthiaCMS.",
  "main": "main.js",
  "author": "me",
  "license": "MIT",
  "dependencies": {
    "an-amazing-dependency": "^2"
  }
}
```

# Example of a `main.js`

```javascript
const join = require("path").join;
const readFileSync = require("fs").readFileSync;
const express = require("express");
const configjson = readFileSync(join(__dirname, "config.json"), { encoding: "utf8", flag: "r" });
const config = JSON.parse(configjson);
module.exports = {
	CyntiaPluginCompat: 1,
    modifyOutputHTML(htmlin) {
		return `${htmlin}
        <!-- This website has my beautiful plugin installed! --!>
        `;
	},
    modifyBodyHTML(htmlin) {
		return `<a href="/amazing">You will see hello world if you click this link!</a>
        ${htmlin}`;
	},
    expressActions(expressapp) {
        expressapp.get('/amazing', (req, res) => {
  res.send('Hello World!')
})
    },
    LogReader(type, msg) {
        if (msg.includes("amazing")) {
            console.log("^ That message contained amazing.")
        }
    }
};
```
