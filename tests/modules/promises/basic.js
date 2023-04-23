

let _resolve;
let _reject;

let promiseWithThen = new Promise((resolve, reject) => {_resolve = resolve; _reject = reject});
let then = promiseWithThen.then(() => {
  console.log("then reached");
}).catch(() => console.log("hey"));
promiseWithThen = then = null;

_reject();
_resolve();

