// Generates the Tauri updater manifest (`latest.json`) for a Windows release.
// The launcher points its updater endpoint at
// `releases/latest/download/latest.json`, so the manifest has to be attached to
// the same GitHub release as the signed installer archive.
import fs from 'node:fs'
import path from 'node:path'

const root = process.cwd()
const bundleDir = path.join(root, 'target/release/bundle/nsis')
const outputDir = path.join(root, 'target/updater')

const version = JSON.parse(
    fs.readFileSync(path.join(root, 'apps/app-frontend/package.json'), 'utf8'),
).version
const repository = process.env.GITHUB_REPOSITORY || 'yuxiaoao/starryisle-Launcher'

if (!fs.existsSync(bundleDir)) {
    throw new Error(`Missing bundle directory: ${bundleDir}`)
}

const entries = fs.readdirSync(bundleDir)
const archives = entries.filter((entry) => entry.endsWith('.nsis.zip'))
if (archives.length !== 1) {
    throw new Error(`Expected exactly one .nsis.zip, found ${archives.length}`)
}

const archive = archives[0]
const signaturePath = path.join(bundleDir, `${archive}.sig`)
if (!fs.existsSync(signaturePath)) {
    throw new Error(`Missing signature for ${archive}`)
}

const signature = fs.readFileSync(signaturePath, 'utf8').trim()
const downloadUrl = `https://github.com/${repository}/releases/download/v${version}/${encodeURIComponent(archive)}`

const manifest = {
    version,
    notes: `StarryIsle Launcher ${version}`,
    pub_date: new Date().toISOString(),
    platforms: {
        'windows-x86_64': {
            signature,
            url: downloadUrl,
        },
    },
}

fs.mkdirSync(outputDir, { recursive: true })
const outputPath = path.join(outputDir, 'latest.json')
fs.writeFileSync(outputPath, `${JSON.stringify(manifest, null, 2)}\n`)

console.log(`Wrote ${outputPath}`)
console.log(JSON.stringify(manifest, null, 2))
