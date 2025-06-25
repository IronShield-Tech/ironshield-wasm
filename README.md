# IronShield WASM

WebAssembly bindings for the IronShield Proof-of-Work system, enabling client-side PoW solving in web browsers.

[![Crates.io](https://img.shields.io/crates/v/ironshield-wasm.svg)](https://crates.io/crates/ironshield-wasm)
[![Documentation](https://docs.rs/ironshield-wasm/badge.svg)](https://docs.rs/ironshield-wasm)
[![License: BUSL-1.1](https://img.shields.io/badge/License-BUSL--1.1-blue.svg)](LICENSE)

## Overview

`ironshield-wasm` provides WebAssembly bindings that allow web browsers and JavaScript environments to participate in IronShield's proof-of-work challenge system. This enables client-side DDoS protection without requiring server-side computation.

## Features

- **Browser Compatible**: Runs in all modern web browsers via WebAssembly
- **JavaScript Integration**: Clean JavaScript API with Promise support
- **Worker Pool Support**: Parallel solving using Web Workers (optional)
- **Mobile Optimized**: Special mobile-safari compatibility mode
- **Memory Efficient**: Optimized for browser memory constraints
- **TypeScript Support**: Full TypeScript definitions included

## Installation

### Using npm/yarn

```bash
npm install ironshield-wasm
# or
yarn add ironshield-wasm
```

### Building from source

```bash
wasm-pack build --target web --out-dir pkg
```

## Quick Start

### Basic JavaScript Usage

```javascript
import init, { solve_challenge, verify_solution } from './pkg/ironshield_wasm.js';

async function main() {
    // Initialize the WASM module
    await init();
    
    // Receive challenge from server (base64url encoded)
    const challengeHeader = "eyJyYW5kb21fbm9uY2UiOi...";
    
    // Solve the challenge
    try {
        const solution = solve_challenge(challengeHeader);
        console.log('Solution found:', solution);
        
        // Send solution back to server
        fetch('/verify', {
            method: 'POST',
            headers: {
                'X-IronShield-Solution': solution
            }
        });
    } catch (error) {
        console.error('Failed to solve challenge:', error);
    }
}

main();
```

### Web Worker Integration

```javascript
// main.js
import { WorkerPool } from './ironshield-worker-pool.js';

const workerPool = new WorkerPool(navigator.hardwareConcurrency);

async function solveWithWorkers(challenge) {
    try {
        const solution = await workerPool.solve(challenge);
        return solution;
    } catch (error) {
        console.error('Worker pool failed:', error);
        throw error;
    }
}
```

### TypeScript Usage

```typescript
import init, { 
    solve_challenge, 
    verify_solution, 
    IronShieldChallenge,
    IronShieldSolution 
} from 'ironshield-wasm';

interface ChallengeResponse {
    challenge: string;
    difficulty: number;
}

async function handleChallenge(response: ChallengeResponse): Promise<string> {
    await init();
    
    const solution: IronShieldSolution = solve_challenge(response.challenge);
    return solution.to_header();
}
```

## API Reference

### Core Functions

#### `solve_challenge(challenge_header: string): IronShieldSolution`
Solves a proof-of-work challenge received as a base64url-encoded header.

- **Parameters**: `challenge_header` - Base64url encoded challenge from server
- **Returns**: Solution object with nonce and verification data
- **Throws**: Error if challenge is invalid or solving fails

#### `verify_solution(challenge: string, solution: string): boolean`
Verifies that a solution is valid for a given challenge.

#### `parse_challenge(header: string): IronShieldChallenge`
Parses a challenge header into a structured challenge object.

### Configuration

#### Feature Flags

- `parallel` (default): Enable Web Worker support for parallel solving
- `no-parallel`: Disable parallel features for compatibility
- `mobile-safari`: Optimized mode for mobile Safari browsers

### Performance Modes

```javascript
// Single-threaded mode (maximum compatibility)
import init, { solve_challenge } from './pkg/ironshield_wasm_no_parallel.js';

// Multi-threaded mode (best performance)
import init, { solve_challenge_parallel } from './pkg/ironshield_wasm.js';
```

## Browser Compatibility

- **Chrome/Edge**: Full support including Web Workers
- **Firefox**: Full support including Web Workers  
- **Safari Desktop**: Full support
- **Safari Mobile**: Use `mobile-safari` feature flag for optimal performance
- **Older Browsers**: Single-threaded mode recommended

## Performance Considerations

### Memory Usage
The WASM module uses approximately 2-4MB of memory during solving.

### CPU Usage
- Single-threaded: Uses one CPU core efficiently
- Multi-threaded: Scales to available CPU cores (up to 8 recommended)

### Battery Impact
Mobile devices will see battery drain during PoW solving. Consider:
- Lower difficulty for mobile users
- Time limits on solving attempts
- Graceful degradation for low-battery devices

## Integration Examples

### React Component

```jsx
import { useEffect, useState } from 'react';
import init, { solve_challenge } from 'ironshield-wasm';

function PoWChallenge({ challenge, onSolved }) {
    const [solving, setSolving] = useState(false);
    
    useEffect(() => {
        if (challenge && !solving) {
            setSolving(true);
            init().then(() => {
                const solution = solve_challenge(challenge);
                onSolved(solution);
                setSolving(false);
            });
        }
    }, [challenge]);
    
    return solving ? <div>Solving challenge...</div> : null;
}
```

### Service Worker

```javascript
// sw.js
importScripts('./pkg/ironshield_wasm.js');

self.addEventListener('message', async (event) => {
    if (event.data.type === 'SOLVE_CHALLENGE') {
        await wasm_bindgen('./pkg/ironshield_wasm_bg.wasm');
        
        try {
            const solution = wasm_bindgen.solve_challenge(event.data.challenge);
            self.postMessage({ type: 'SOLUTION', solution });
        } catch (error) {
            self.postMessage({ type: 'ERROR', error: error.message });
        }
    }
});
```

## License

This project is licensed under the [Business Source License 1.1](LICENSE). 
It will automatically convert to Apache-2.0 on July 24, 2028.

## Contributing

See the main [IronShield repository](https://github.com/IronShield-Tech/IronShield) for contribution guidelines. 