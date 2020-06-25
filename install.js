const { Binary } = require('binary-install')
const os = require('os')
const fs = require('fs')
const { join } = require('path')
const { version, name, repository } = require('./package.json')

const supportedPlatforms = [
  {
    TYPE: 'Linux',
    ARCHITECTURE: 'x64',
    RUST_TARGET: 'x86_64-unknown-linux-musl'
  },
  {
    TYPE: 'Darwin',
    ARCHITECTURE: 'x64',
    RUST_TARGET: 'x86_64-apple-darwin'
  }
]

const platform = () => {
  const type = os.type()
  const architecture = os.arch()
  for (let index in supportedPlatforms) {
    let supportedPlatform = supportedPlatforms[index]
    if (
      type === supportedPlatform.TYPE &&
      architecture === supportedPlatform.ARCHITECTURE
    ) {
      return supportedPlatform.RUST_TARGET
    }
  }

  throw new Error(
    `Platform with type "${type}" and architecture "${architecture}" is not supported by ${name}.\nYour system must be one of the following:\n\n${supportedPlatforms.map(
      sp => sp.RUST_TARGET
    )}`
  )
}

const install = async () => {
  const target = platform()
  const url = `${repository.url}/releases/download/v${version}/${target}.tar.gz`
  const temp = os.tmpdir() + '/roomservice'
  const binary = new Binary(url, {
    installDirectory: temp,
    name: 'roomservice'
  })
  
  await binary.install()

  // Sigh.
  await new Promise(res => setTimeout(res, 3000))

  fs.renameSync(temp + '/bin/' + target+ '/roomservice', '/usr/local/bin/roomservice')
}

install()
