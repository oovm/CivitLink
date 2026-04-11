const { execSync } = require('child_process');
try {
  const output = execSync('cargo check -p gg-runtime-core --no-default-features', {
    cwd: 'e:\\灵之镜有限公司\\gg-game-engine',
    encoding: 'utf8',
    timeout: 300000,
    stdio: ['pipe', 'pipe', 'pipe']
  });
  console.log('STDOUT:', output);
} catch (e) {
  console.log('EXIT CODE:', e.status);
  console.log('STDOUT:', e.stdout);
  console.log('STDERR:', e.stderr.substring(e.stderr.length - 3000));
}
