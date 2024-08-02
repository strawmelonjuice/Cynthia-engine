# Plugin API's

# JS/TS

Currently there is only the JS/TS API available. Most important change you should is that Cynthia `v3` communicates with it's plugins over `STDIO`. In Cynthia `v2` this was done through calling arguments and local HTTP ports, causing loads of overhead.

Cynthia `v3` aims to bring comfortable communication with it's plugins back to the levels of `v0/ts-draft`, where a Node plugin would be imported and called as a library, then called with an object, on which methods were available to control Cynthia's behaviour.

The Node package for writing Cynthia plugins can be found at:

<https://www.npmjs.com/package/@cynthiaweb/plugin-api>




