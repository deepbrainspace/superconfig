# SuperFigment Encryption Architecture Design

## Overview

SuperFigment's encryption feature will provide automatic encryption/decryption of sensitive configuration values, enabling safe git storage while maintaining the library's core strengths: hierarchical configuration, intelligent array merging, and cross-language WASM compatibility.

## Core Design Principles

1. **Zero-Configuration Bootstrap**: SuperFigment configures its own encryption without external config files
2. **Transparent Operations**: Encryption/decryption happens automatically during config loading
3. **Git-Safe Storage**: Encrypted values can be safely committed to version control
4. **Cross-Language Compatibility**: Full functionality available via WASM bindings
5. **Selective Encryption**: Only sensitive values are encrypted, not entire config files

## Encryption Methods

### 1. Pattern-Based Auto-Detection

```rust
// Automatically encrypt values matching sensitive patterns
let config = SuperFigment::new()
    .with_auto_encryption(true)
    .with_file("config.toml");

// In config.toml:
// password = "secret123"           -> gets encrypted automatically
// api_key = "sk-1234567890"        -> gets encrypted automatically
// database_host = "localhost"      -> stays plaintext
```

### 2. Explicit Annotation

```rust
// Use special suffix to mark values for encryption
// In config.toml:
// password_encrypted = "secret123"     -> gets encrypted
// api_key_secret = "sk-1234567890"     -> gets encrypted
// normal_value = "plaintext"           -> stays plaintext
```

### 3. Schema-Driven Encryption

```rust
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize)]
struct Config {
    #[superfigment(encrypt)]
    database_password: String,

    #[superfigment(encrypt)]
    api_keys: Vec<String>,

    // Normal field - not encrypted
    database_host: String,
}
```

## Key Management Strategy

### Multi-User Asymmetric Encryption

SuperFigment uses **hybrid encryption** (RSA + AES) to support teams with different access levels:

```rust
// Each encrypted value uses:
// 1. Random AES-256 key (for actual encryption)
// 2. AES key encrypted with each authorized user's RSA public key
// 3. Stored format: ENC[recipient1:encrypted_aes_key1,recipient2:encrypted_aes_key2:aes_encrypted_data]
```

### Key Architecture

```toml
# .superfigment/recipients
[users]
alice = "ssh-rsa AAAAB3NzaC1yc2E... alice@company.com"
bob = "ssh-rsa AAAAB3NzaC1yc2E... bob@company.com"
deployment = "ssh-rsa AAAAB3NzaC1yc2E... deployment@ci.company.com"

[groups]
developers = ["alice", "bob"]
production = ["alice", "deployment"]
```

### Encryption Process

```rust
// When encrypting a value:
// 1. Generate random AES-256 key
let aes_key = generate_random_aes_key();

// 2. Encrypt value with AES key
let encrypted_data = aes_encrypt(plaintext, &aes_key);

// 3. Encrypt AES key for each recipient
let mut encrypted_keys = Vec::new();
for recipient in recipients {
    let encrypted_aes = rsa_encrypt(&aes_key, &recipient.public_key);
    encrypted_keys.push((recipient.name, encrypted_aes));
}

// 4. Store in format: ENC[alice:key1,bob:key2:encrypted_data]
let final_value = format!("ENC[{}:{}]",
    encrypted_keys.join(","),
    base64_encode(encrypted_data)
);
```

### Decryption Process

```rust
// When decrypting a value:
// 1. Parse encrypted format
let (recipients_keys, encrypted_data) = parse_encrypted_value(encrypted_value);

// 2. Find matching recipient (using SSH key or identity)
let current_user = identify_current_user(); // Uses SSH agent, GPG, etc.
let encrypted_aes_key = recipients_keys.get(current_user)?;

// 3. Decrypt AES key with user's private key
let aes_key = rsa_decrypt(encrypted_aes_key, &current_user.private_key);

// 4. Decrypt actual data
let plaintext = aes_decrypt(encrypted_data, &aes_key);
```

### Bootstrap Solution: Identity-First Hierarchy

```rust
// SuperFigment automatically identifies current user and finds keys
let encryption_config = SuperFigment::bootstrap()
    .with_ssh_agent()                    // 1. SSH agent keys (highest priority)
    .with_gpg_keys()                     // 2. GPG keys
    .with_env("SUPERFIGMENT_")           // 3. Environment variables
    .with_keyfiles()                     // 4. Local key files
    .with_defaults(default_encryption)   // 5. Development fallback
    .extract::<EncryptionConfig>()?;
```

### Identity Sources (in priority order):

1. **SSH Agent** (most common for developers)

   ```bash
   # Uses existing SSH keys from ssh-agent
   ssh-add -l  # Lists available keys
   # SuperFigment matches against .superfigment/recipients
   ```

2. **GPG Keys**

   ```bash
   # Uses GPG keyring for encryption/decryption
   gpg --list-secret-keys
   ```

3. **Environment Variables** (for CI/CD)

   ```bash
   SUPERFIGMENT_PRIVATE_KEY="-----BEGIN PRIVATE KEY-----..."
   SUPERFIGMENT_KEY_ID="deployment"
   ```

4. **Key Files** (local development)
   ```bash
   ~/.superfigment/private.pem
   ./.superfigment/private.pem
   ```

## File Format Design

### Encrypted Value Format

```toml
# Original config.toml
database_url = "postgres://user:pass@localhost/db"
api_key = "sk-1234567890abcdef"
features = ["auth", "logging"]
log_level = "info"

# After encryption (multi-user format)
database_url = "ENC[alice:RSA_encrypted_AES_key1,bob:RSA_encrypted_AES_key2:AES_encrypted_data]"
api_key = "ENC[alice:different_AES_key1,deployment:different_AES_key2:different_AES_data]"
features = "ENC[alice:array_AES_key1,bob:array_AES_key2:encrypted_json_array]"
log_level = "info" # Not encrypted - not sensitive
```

### Array Merging with Encryption

```toml
# Base config (encrypted arrays work with _add/_remove)
features = "ENC[alice:key1,bob:key2:encrypted_base_array]"

# Override config
features_add = "ENC[alice:key3,bob:key4:encrypted_additions]"
features_remove = "ENC[alice:key5,bob:key6:encrypted_removals]"

# SuperFigment process:
# 1. Decrypt all three arrays to plaintext
# 2. Apply intelligent merging: base + additions - removals
# 3. Application receives final merged plaintext array
```

### Metadata Preservation

```toml
# SuperFigment preserves all formatting and comments
[database]
# Production database configuration
url = "ENC[alice:key1,deployment:key2:encrypted_connection_string]"
timeout = 30 # seconds

# API Configuration
[api]
key = "ENC[developers:key3,production:key4:encrypted_api_key]"
rate_limit = 1000
```

## Implementation Architecture

### Core Components

```rust
// Core encryption interface
pub struct EncryptionProvider {
    cipher: Box<dyn Cipher>,
    key_manager: Box<dyn KeyManager>,
    pattern_matcher: PatternMatcher,
}

pub trait Cipher {
    fn encrypt(&self, plaintext: &str, key: &[u8]) -> Result<String, Error>;
    fn decrypt(&self, ciphertext: &str, key: &[u8]) -> Result<String, Error>;
}

pub trait KeyManager {
    fn get_master_key(&self) -> Result<Vec<u8>, Error>;
    fn derive_value_key(&self, master_key: &[u8], context: &str) -> Result<Vec<u8>, Error>;
}

// Integration with SuperFigment
impl SuperFigment {
    pub fn with_auto_encryption(mut self, enabled: bool) -> Self {
        self.encryption_enabled = enabled;
        self
    }

    pub fn with_encryption_patterns(mut self, patterns: Vec<String>) -> Self {
        self.encryption_patterns = patterns;
        self
    }
}
```

### Encryption Flow

```rust
// Internal processing during config loading
impl SuperFigment {
    fn process_provider(&self, provider: Box<dyn Provider>) -> Result<Figment, Error> {
        if !self.encryption_enabled {
            return Ok(self.figment.merge(provider));
        }

        // 1. Load raw config data
        let raw_data = provider.data()?;

        // 2. Process each value
        let processed_data = self.process_encryption(raw_data)?;

        // 3. Create provider with processed data
        let encrypted_provider = ProcessedProvider::new(processed_data);

        Ok(self.figment.merge(encrypted_provider))
    }

    fn process_encryption(&self, data: figment::value::Map) -> Result<figment::value::Map, Error> {
        let mut result = figment::value::Map::new();

        for (key, value) in data {
            let processed_value = if self.should_encrypt(&key, &value)? {
                self.encrypt_value(&key, &value)?
            } else if self.is_encrypted(&value)? {
                self.decrypt_value(&key, &value)?
            } else {
                value
            };

            result.insert(key, processed_value);
        }

        Ok(result)
    }
}
```

## WASM Compatibility

### Cross-Language API Design

The encryption functionality maintains identical APIs across all languages:

```typescript
// TypeScript (via WASM)
import { SuperFigment } from "@superfigment/wasm";

const config = new SuperFigment()
  .withAutoEncryption(true)
  .withFile("config.toml")
  .extract<AppConfig>();
```

```python
# Python (via WASM)
from superfigment import SuperFigment

config = (SuperFigment()
    .with_auto_encryption(True)
    .with_file('config.toml')
    .extract(AppConfig))
```

```go
// Go (via WASM)
import "github.com/superfigment/go-superfigment"

config := superfigment.New().
    WithAutoEncryption(true).
    WithFile("config.toml").
    Extract(&AppConfig{})
```

### Key Management Across Languages

Since WASM has access to environment variables and file system (via WASI), the key management works identically:

```rust
// Internal WASM implementation (shared across all languages)
#[wasm_bindgen]
pub struct WasmSuperFigment {
    inner: SuperFigment,
}

#[wasm_bindgen]
impl WasmSuperFigment {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Self {
            inner: SuperFigment::new()
        }
    }

    #[wasm_bindgen(js_name = withAutoEncryption)]
    pub fn with_auto_encryption(mut self, enabled: bool) -> Self {
        self.inner = self.inner.with_auto_encryption(enabled);
        self
    }
}
```

## Security Considerations

### Encryption Standards

- **Algorithm**: AES-256-GCM for authenticated encryption
- **Key Derivation**: PBKDF2 with 100,000 iterations + salt
- **Nonce Generation**: Cryptographically secure random per value
- **Key Storage**: Never persisted to disk without explicit user action

### Threat Model Protection

- ✅ **Git Repository Exposure**: Encrypted values are safe in public repos
- ✅ **Config File Theft**: Individual files contain no usable secrets
- ✅ **Memory Dumps**: Keys are cleared after use
- ✅ **Log Exposure**: Decrypted values never appear in logs
- ⚠️ **Runtime Memory**: Decrypted values exist in application memory
- ⚠️ **Key Compromise**: If master key is compromised, all values are at risk

### Development vs Production

```rust
// Development mode (convenient but less secure)
let config = SuperFigment::new()
    .with_auto_encryption(true)
    .with_dev_key_derivation(true)  // Derives from project path
    .with_file("config.toml");

// Production mode (secure key management required)
let config = SuperFigment::new()
    .with_auto_encryption(true)
    .with_env("SUPERFIGMENT_")      // Requires SUPERFIGMENT_MASTER_KEY
    .with_file("config.toml");
```

## CLI Tools Integration

### Encryption Commands

```bash
# Encrypt sensitive values in existing config
superfigment encrypt config.toml

# Decrypt for editing (temporary plaintext)
superfigment decrypt config.toml --edit

# Generate new master key
superfigment keygen > ~/.superfigment/master.key

# Validate encrypted config
superfigment validate config.toml
```

### Git Integration

```bash
# Git clean filter (automatic encryption on commit)
echo "*.toml filter=superfigment" >> .gitattributes
git config filter.superfigment.clean 'superfigment encrypt --stdin'
git config filter.superfigment.smudge 'superfigment decrypt --stdin'
```

## Migration Strategy

### Phase 1: Core Implementation

- [x] Research existing solutions
- [x] Architecture design
- [ ] Core encryption/decryption engine
- [ ] Bootstrap key management
- [ ] Pattern-based value detection

### Phase 2: SuperFigment Integration

- [ ] Provider chain integration
- [ ] Auto-encryption during config loading
- [ ] Error handling and validation
- [ ] Comprehensive test suite

### Phase 3: WASM & Cross-Language

- [ ] WASM bindings for encryption
- [ ] TypeScript wrapper library
- [ ] Python wrapper library
- [ ] Go wrapper library

### Phase 4: Developer Experience

- [ ] CLI tools for key management
- [ ] Git integration utilities
- [ ] IDE extensions for encrypted value editing
- [ ] Documentation and examples

## Competitive Advantages

1. **Zero External Dependencies**: No separate encryption tools required
2. **Language Agnostic**: Same encryption API across all supported languages
3. **Intelligent Detection**: Automatic identification of sensitive values
4. **Git Native**: Seamless integration with existing git workflows
5. **Bootstrap Self-Configuration**: No chicken-and-egg configuration problems
6. **Hierarchical Aware**: Encryption works with SuperFigment's config hierarchy
7. **Array Merge Compatible**: Encrypted arrays still support `_add`/`_remove` operations

This design positions SuperFigment as the first configuration management library to provide built-in encryption with true cross-language compatibility via WASM.
