import { spawnSync } from 'node:child_process';
import { mkdir, copyFile, cp, readFile, writeFile, readdir } from 'node:fs/promises';
import { homedir } from 'node:os';
import { join } from 'node:path';

const env = { ...process.env, PATH: `${join(homedir(), '.cargo/bin')}:${process.env.PATH}` };
const r = spawnSync('cargo', ['build', '--locked', '--release', '--target', 'wasm32-unknown-unknown'], { env, stdio: 'inherit' });
if (r.status !== 0) process.exit(r.status || 1);
await mkdir('dist', { recursive: true });
await cp('web', 'dist', { recursive: true });
await copyFile('target/wasm32-unknown-unknown/release/beach-buggy.wasm', 'dist/beach-buggy.wasm');
// Vendor the exact loader shipped with the locked miniquad dependency.
const lock = await readFile('Cargo.lock', 'utf8');
const version = lock.match(/name = "miniquad"\nversion = "([^"]+)"/)[1];
const sourceRoot = join(homedir(), '.cargo/registry/src');
let loader;
for (const root of await readdir(sourceRoot)) {
  try { loader = await readFile(join(sourceRoot, root, `miniquad-${version}`, 'js/gl.js')); break; } catch {}
}
if (!loader) throw new Error(`Cannot locate miniquad ${version} loader`);
await writeFile('dist/gl.js', loader);
// Preserve notices for source embedded in the distributable WASM and loader.
const metadataRun = spawnSync('cargo', ['metadata', '--locked', '--format-version', '1'], {env,encoding:'utf8'});
if(metadataRun.status!==0)throw new Error('Cannot gather dependency license notices');
const metadata=JSON.parse(metadataRun.stdout);
const notices=[];
for(const pkg of metadata.packages.filter(p=>p.source)) {
  const folder=join(pkg.manifest_path,'..');
  const destination=join('dist','licenses',`${pkg.name}-${pkg.version}`);
  await mkdir(destination,{recursive:true});
  for(const name of await readdir(folder)) {
    if(/^(licen[cs]e|copying|notice)/i.test(name))await cp(join(folder,name),join(destination,name),{recursive:true});
  }
  notices.push(`${pkg.name} ${pkg.version} — ${pkg.license || 'See crate notices'} — ${pkg.repository || 'https://crates.io/crates/'+pkg.name}`);
}
await copyFile('LICENSE','dist/LICENSE');
await writeFile('dist/THIRD_PARTY_NOTICES.txt',notices.join('\n')+'\n');
console.log('Built dist/ — start with npm run dev');
