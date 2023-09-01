const { Console } = console;

let fs = require("node:fs");

const stdout = fs.createWriteStream('./stdout.log');
const stderr = fs.createWriteStream('./stderr.log');

console.assert(false);
console.assert(false, "assert failed");
console.assert(true, "not printed");

let c = new Console(stdout, stderr);
c.assert(false, "c: assert failed");
c.assert(true, "c: not printed");
