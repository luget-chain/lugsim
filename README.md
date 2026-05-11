# LUGET Research Simulator

[![CI](https://github.com/luget-chain/lugsim/actions/workflows/ci.yml/badge.svg)](https://github.com/luget-chain/lugsim/actions)
[![Tests](https://img.shields.io/badge/tests-66%20passed-brightgreen)](https://github.com/luget-chain/lugsim/actions)
[![Version](https://img.shields.io/badge/version-0.9.0-orange)](https://github.com/luget-chain/lugsim/releases)
[![Rust](https://img.shields.io/badge/rust-1.78%2B-blue)](https://www.rust-lang.org)

**LUGET** is a dual-domain Layer-1 blockchain where sound money (UTXO Core) and safe programmability (Object VM) are architecturally separated by a cryptographically firewalled bridge.

**No premine. No VCs. No insider tokens.** Built from a village in Kenya.

---

## Quick Start

```bash
git clone https://github.com/luget-chain/lugsim.git
cd lugsim
cargo build --all
cargo test --all
docker build -t lugsim .
docker run lugsim

---

### Task 3: `luget.org` landing page

```bash
mkdir -p ~/luget-landing
cat > ~/luget-landing/index.html << 'ENDOFFILE'
<!DOCTYPE html>
<html lang="en">
<head>
<meta charset="UTF-8">
<meta name="viewport" content="width=device-width, initial-scale=1.0">
<title>LUGET — Sound Money. Safe Programmability. One Chain. No Insiders.</title>
<style>
* { margin: 0; padding: 0; box-sizing: border-box; }
body { background: #06060d; color: #d0d0d0; font-family: 'Courier New', monospace; min-height: 100vh; display: flex; flex-direction: column; align-items: center; justify-content: center; text-align: center; padding: 20px; }
h1 { color: #ff5f1f; font-size: 3.5em; letter-spacing: 8px; margin-bottom: 10px; }
.tagline { color: #999; font-size: 1.1em; margin-bottom: 40px; max-width: 600px; line-height: 1.6; }
.buttons { display: flex; flex-wrap: wrap; gap: 15px; justify-content: center; margin-bottom: 50px; }
.btn { border: 1px solid #ff5f1f; color: #ff5f1f; padding: 12px 28px; text-decoration: none; font-size: 0.9em; letter-spacing: 2px; transition: all 0.3s; }
.btn:hover { background: #ff5f1f; color: #06060d; }
.stats { display: flex; flex-wrap: wrap; gap: 20px; justify-content: center; margin-bottom: 50px; }
.stat { text-align: center; min-width: 100px; }
.stat .num { color: #ff5f1f; font-size: 1.8em; font-weight: bold; }
.stat .lbl { color: #777; font-size: 0.7em; text-transform: uppercase; letter-spacing: 1px; }
.origin { color: #555; font-size: 0.8em; margin-top: 40px; border-top: 1px solid #1a1a1a; padding-top: 20px; max-width: 500px; line-height: 1.8; }
@media (max-width: 600px) { h1 { font-size: 2.2em; } }
</style>
</head>
<body>
<h1>LUGET</h1>
<p class="tagline">Sound Money. Safe Programmability.<br>One Chain. No Insiders.</p>

<div class="stats">
<div class="stat"><div class="num">66</div><div class="lbl">Tests</div></div>
<div class="stat"><div class="num">0</div><div class="lbl">Failures</div></div>
<div class="stat"><div class="num">100M</div><div class="lbl">Max Supply</div></div>
<div class="stat"><div class="num">0%</div><div class="lbl">Insider Allocation</div></div>
</div>

<div class="buttons">
<a href="https://luget-chain.github.io/litepaper" class="btn">Litepaper</a>
<a href="https://github.com/luget-chain/lugsim" class="btn">GitHub</a>
<a href="https://luget-chain.github.io/lugsim/explorer/" class="btn">Explorer</a>
<a href="https://discord.gg/M2f34jdP5M" class="btn">Discord</a>
</div>

<p class="origin">
Built from Kenya<br>
by Kipngetich Clinton — a civil engineering graduate with a phone and a conviction.
</p>
</body>
</html>
