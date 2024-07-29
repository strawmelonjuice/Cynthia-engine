/*
 * Copyright (c) 2024, MLC 'Strawmelonjuice' Bloeiman
 *
 * Licensed under the GNU AFFERO GENERAL PUBLIC LICENSE Version 3, see the LICENSE file for more information.
 */

console.log(`
Dit wordt gewoon naar de console geschreven.
Maar dit:
send:    {
send:        "test": "Wat de fuck",
send:        "geel": [
send:        "verf", "krijt", "verf"
send:        ]
send:    }
Is als het goed is nu een struct.
`);
console.log(`
En dit is weer gewoon json.
    {
        "test": "Wat de fuck",
        "geel": [
        "verf", "krijt", "verf"
        ]
    }
`);
console.log("En nu in stukjes:");
console.log(`
send:    {
send:        "test": "Wat de fuck",
`);
console.log("send:        \"geel\": [")
console.log(`
send:        "verf", "krijt", "verf"
send:        ]
send:    }`);
console.log("Ziezo, in elkaar gezet!");