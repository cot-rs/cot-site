const { spawnSync } = require('node:child_process')
const fs = require('node:fs')
const path = require('node:path')

const testFiles = fs.readdirSync(__dirname)
  .filter(file => file.endsWith('.cjs') && file !== 'run.cjs')
  .sort()

if (testFiles.length === 0) throw new Error('No integration tests found')

for (const file of testFiles) {
  console.log(`\nRunning integration test: ${file}`)
  const result = spawnSync(process.execPath, [path.join(__dirname, file), ...process.argv.slice(2)], {
    stdio: 'inherit',
    env: process.env
  })
  if (result.error) throw result.error
  if (result.status !== 0) process.exit(result.status || 1)
}
