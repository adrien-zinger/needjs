const { open } = require("node:fs/promises");

open("test.txt").then(() => console.log("success read"));