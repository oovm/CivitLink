import { execSync } from 'child_process';
import { join } from 'path';

const examples = [
  'config-basic',
  'effect-advanced', 
  'database-basic', 
  'auth-basic', 
  'cache-basic', 
  'observability-basic',
  'effect-basic',
  'https-basic',
  'error-handling'
];

const waeRoot = 'e:\\\\wae';

console.log('Checking all examples for compilation...\n');

for (const example of examples) {
  const examplePath = join(waeRoot, 'examples', example);
  console.log(`Checking ${example}...`);
  
  try {
    execSync(`cargo check`, {
      cwd: examplePath,
      stdio: 'inherit',
      encoding: 'utf-8'
    });
    console.log(`✓ ${example} compiled successfully!\n`);
  } catch (error) {
    console.error(`✗ ${example} failed to compile!\n`);
  }
}

console.log('Done!');
