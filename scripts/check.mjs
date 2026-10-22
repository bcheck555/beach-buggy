import { spawnSync } from 'node:child_process';
import { homedir } from 'node:os';
import { join } from 'node:path';
const env={...process.env,PATH:`${join(homedir(),'.cargo/bin')}:${process.env.PATH}`};
for(const args of [
  ['fmt','--check'],
  ['clippy','--locked','--all-targets','--','-D','warnings'],
  ['clippy','--locked','--target','wasm32-unknown-unknown','--','-D','warnings'],
  ['test','--locked','--release','--lib'],
  ['test','--locked','--release','--bin','beach-buggy'],
]) {
  console.log(`\nChecking: cargo ${args.join(' ')}`);
  const r=spawnSync('cargo',args,{env,stdio:'inherit'});
  if(r.status!==0)process.exit(r.status||1);
}
for(const file of ['web/bridge.js','scripts/build.mjs','scripts/serve.mjs','scripts/setup.mjs','tests/browser.spec.js']){
  const r=spawnSync(process.execPath,['--check',file],{stdio:'inherit'});
  if(r.status!==0)process.exit(r.status||1);
}
console.log('\nRust and JavaScript checks passed. Run npm run build and browser tests next.');
