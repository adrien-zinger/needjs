//console.log("hello %s!", "world");
//console.log("hello %s", "world", "!");
//let a = null;
//console.log("hello %s", "world", "!", a);
//
//console.log("%i", "5,6");
//let n = Date.now();
//console.log("", n);

let fs = require("node:fs");

const stdout = fs.createWriteStream('./stdout.log');
const stderr = fs.createWriteStream('./stderr.log');

const { Console } = console;
let c = new Console(stdout, stderr, false);

console.log(`test: ${c}`);
c.log("hello world!!");
c.error("error logged");
