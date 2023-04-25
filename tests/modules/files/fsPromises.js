const { open, access, constants } = require("node:fs/promises");

open("test.txt").then(() => console.log("success read"));

console.log(constants)
