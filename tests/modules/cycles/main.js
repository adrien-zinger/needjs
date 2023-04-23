console.log('main starting');
console.log("" + Object.getOwnPropertyNames(console.prototype).join("-"))
const a = require('./a.js');
const b = require('./b.js');
console.log('in main, a.done = ' + a.done + ', b.done = ' + b.done);