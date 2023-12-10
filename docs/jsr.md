# Installing a JS runtime

_Cynthia doesn't need one, but most of it's plugins do!_

## Supported runtimes
- Cynthia supports **_Node.JS_ everywhere**, and **_Bun_ on Linux and MacOS**. If you have Bun working on Windows through WSL, it'll try that but most likely fail (and fall back on Node.JS).

- Any available Bun instance is prefered over Node.JS by Cynthia, this because Cynthia assumes Bun will start faster.
    - Cynthia (plv2) plugins are rapidly iterated over, giving each a chance to modify or enhance outputs. This makes starting times **very** important, as a single action might mean 30 scripts being ran and returned from. This said, in testing, no serious issues have been seen, yet.
- Cynthia finds runtimes through the _path_.

## Downloads
I'll link you to these, and you should be able to download and install them according to their own websites.
- Node.JS: <https://nodejs.org/en/download/> (LTS / v20.x.x is used in tests, too.)
- Bun: <https://bun.sh>