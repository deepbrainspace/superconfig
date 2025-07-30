# Using SuperConfig in Next.js Applications

SuperConfig now supports **three WASM variants** for different Next.js use cases:

## 🎯 WASM Variants Available

### 1. **Browser WASM** (41KB) - Client-Side

```bash
wasm-pack build --target web --features wasm
```

- **Usage**: Client-side React components
- **Limitations**: No filesystem access, verbosity changes limited
- **Perfect for**: Static configuration, basic settings

### 2. **Node.js WASM** (41KB) - Server-Side

```bash
wasm-pack build --target nodejs --features wasm
```

- **Usage**: Next.js API routes, middleware
- **Limitations**: No filesystem access, but works in Node.js runtime
- **Perfect for**: Server-side basic configuration access

### 3. **WASI WASM** (50KB) - Full Filesystem Access

```bash
cargo build --target wasm32-wasip1 --features wasm --release
```

- **Usage**: Next.js server-side with full SuperConfig features
- **Capabilities**: Complete filesystem access, full verbosity system
- **Perfect for**: Server-side configuration loading, file-based configs

## 📁 Next.js Integration Examples

### Client-Side Usage (Browser WASM)

```javascript
// pages/settings.js
import { SuperConfig } from '../pkg/superconfig_ffi.js';

export default function SettingsPage() {
  const [config, setConfig] = useState(null);
  
  useEffect(() => {
    const loadConfig = async () => {
      const superConfig = new SuperConfig();
      const verbosity = superConfig.get_verbosity();
      setConfig({ verbosity });
      superConfig.free(); // Clean up WASM memory
    };
    loadConfig();
  }, []);
  
  return <div>Verbosity: {config?.verbosity}</div>;
}
```

### Server-Side API Route (WASI WASM)

```javascript
// pages/api/config.js
import { readFile } from 'fs/promises';
import { WASI } from 'wasi';

export default async function handler(req, res) {
  // Load WASI SuperConfig with filesystem access
  const wasi = new WASI({
    version: 'preview1',
    preopens: { '/config': './config' }, // Mount config directory
  });
  
  const wasmBuffer = await readFile('./superconfig_ffi.wasm');
  const wasmModule = await WebAssembly.compile(wasmBuffer);
  const wasmInstance = await WebAssembly.instantiate(wasmModule, {
    wasi_snapshot_preview1: wasi.wasiImport,
  });
  
  wasi.initialize(wasmInstance);
  
  // Now SuperConfig can read files from /config directory
  res.json({ status: 'SuperConfig loaded with file access!' });
}
```

### Middleware Usage

```javascript
// middleware.js
import { NextResponse } from 'next/server';

export async function middleware(request) {
  // Use Node.js WASM version in middleware
  const { SuperConfig } = await import('./pkg/superconfig_ffi.js');
  
  const config = new SuperConfig();
  const verbosity = config.get_verbosity();
  
  const response = NextResponse.next();
  response.headers.set('X-Config-Verbosity', verbosity.toString());
  
  config.free();
  return response;
}
```

## 🚀 Build Commands Summary

| Target  | Command                                                        | Size | Filesystem | Use Case                  |
| ------- | -------------------------------------------------------------- | ---- | ---------- | ------------------------- |
| Browser | `wasm-pack build --target web --features wasm`                 | 41KB | ❌         | Client-side React         |
| Node.js | `wasm-pack build --target nodejs --features wasm`              | 41KB | ❌         | API routes, middleware    |
| WASI    | `cargo build --target wasm32-wasip1 --features wasm --release` | 50KB | ✅         | Full server-side features |

## 💡 Recommendations

- **Client-side**: Use Browser WASM for basic configuration display
- **API Routes**: Use WASI WASM for full configuration file loading
- **Middleware**: Use Node.js WASM for lightweight configuration access
- **Production**: WASI WASM gives you complete SuperConfig functionality in Next.js server environment

The WASI version is the most powerful and gives you access to SuperConfig's full feature set including file loading, hierarchical configuration, and the complete verbosity system!
