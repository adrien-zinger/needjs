new Promise(resolve => resolve()).then(() => console.log("a resolved"));
const b = require("./b");
console.log("a loaded");
