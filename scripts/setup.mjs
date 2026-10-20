import { mkdir, writeFile, chmod } from 'node:fs/promises';
import { spawnSync } from 'node:child_process';
import { homedir } from 'node:os';
import { join } from 'node:path';

const cargo = join(homedir(), '.cargo/bin');
const env = { ...process.env, PATH: `${cargo}:${process.env.PATH}` };
function run(command, args) {
  const r = spawnSync(command, args, { env, stdio: 'inherit' });
  if (r.status !== 0) throw new Error(`${command} failed (${r.status})`);
}
if (spawnSync('rustup', ['--version'], { env, stdio: 'ignore' }).status !== 0) {
  if (process.platform !== 'linux' || process.arch !== 'x64') {
    throw new Error('Install Rust using rustup.rs, then run npm run setup again.');
  }
  await mkdir('.tools', { recursive: true });
  const response = await fetch('https://static.rust-lang.org/rustup/dist/x86_64-unknown-linux-gnu/rustup-init');
  if (!response.ok) throw new Error(`Rust download: ${response.status}`);
  await writeFile('.tools/rustup-init', new Uint8Array(await response.arrayBuffer()));
  await chmod('.tools/rustup-init', 0o755);
  run('./.tools/rustup-init', ['-y', '--profile', 'minimal', '--default-toolchain', '1.90.0', '--no-modify-path']);
}
run('rustup', ['toolchain', 'install', '1.90.0', '--profile', 'minimal', '--component', 'rustfmt,clippy']);
run('rustup', ['target', 'add', 'wasm32-unknown-unknown', '--toolchain', '1.90.0']);
console.log('Rust ready. Add ~/.cargo/bin to PATH for direct cargo commands.');
