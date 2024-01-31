# Installing a JS runtime

_Cynthia doesn't need one, but most of its plugins do!_

## Supported by Cynthia

#### Runtimes

Cynthia supports **_Node.js_ and _Bun_ everywhere**. If you have Bun working on Windows through WSL, it'll try that but most likely fail (and fall back on Node.js).

- Any available Bun instance is prefered over Node.js by Cynthia, this because Cynthia assumes Bun will start faster.
    - Cynthia (plv2) plugins are rapidly iterated over, giving each a chance to modify or enhance outputs. This makes starting times **very** important, as a single action might mean 30 scripts being run and returned from. This said, in testing, no serious issues have been seen, yet.
- Cynthia finds runtimes through the _path_.

#### Package managers
Cynthia tries to support the package managers provided with their respective runtimes, however, will also add support for _PNPM_ in the future. 

### Order of preference

Cynthia will have a preference when it finds multiple supported components installed. These are different for package managers than for runtimes.

| **_Runtime_** | **_Package manager_** |
|---------------|-----------------------|
| Bun           | _PNPM_                |
| Node.JS       | Bun                   |
|               | NPM                   |



## Downloads
I'll link you to these, and you should be able to download and install them according to their own websites.
- Node.js: <https://nodejs.org/en/download/> (`LTS / v20.x.x` is recommended.)
- Bun: <https://bun.sh> (`current` is used in tests, too.)