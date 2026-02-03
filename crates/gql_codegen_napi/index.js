const { platform, arch } = process;

let nativeBinding = null;

const platformArch = `${platform}-${arch}`;

switch (platformArch) {
  case 'darwin-arm64':
    nativeBinding = require('./sgc-core.darwin-arm64.node');
    break;
  case 'darwin-x64':
    nativeBinding = require('./sgc-core.darwin-x64.node');
    break;
  case 'linux-x64':
    nativeBinding = require('./sgc-core.linux-x64-gnu.node');
    break;
  case 'linux-arm64':
    nativeBinding = require('./sgc-core.linux-arm64-gnu.node');
    break;
  case 'win32-x64':
    nativeBinding = require('./sgc-core.win32-x64-msvc.node');
    break;
  default:
    throw new Error(`Unsupported platform: ${platformArch}`);
}

module.exports = nativeBinding;
