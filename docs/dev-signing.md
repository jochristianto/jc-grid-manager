# Dev code signing (macOS)

Keeps local macOS builds signed with a stable identity so macOS doesn't forget
the Accessibility permission grant every time you rebuild.

- **Scope:** dev-time only. Nothing here is release signing or notarization —
  that stays out of scope for v1 (`docs/idea.md` §3, §5.6).
- **Bundle identifier:** `com.jochristianto.jcgridmanager`
- **Expected certificate name:** `JC Grid Manager Dev`

## Why this is needed

macOS ties the Accessibility permission (System Settings → Privacy & Security
→ Accessibility) to the app's *code identity*, not just its bundle ID. The app
ships unsigned in v1 (`docs/idea.md` §5.6 — no paid Apple Developer ID), and
an unsigned/ad-hoc build gets a fresh, content-based signature on every
compile. macOS reads that as "a different app" each time, so the Accessibility
grant disappears and you have to re-grant it constantly during development.

The fix is free and local: sign dev builds with a **self-signed code-signing
certificate** that never changes. As long as every build is signed with the
same certificate, macOS keeps recognizing it as the app you already trusted,
even though the binary's contents change on every build.

## 1. Create the certificate (one-time, per machine)

Do this once on each Mac you develop on. GUI and CLI produce an equivalent
result — pick whichever you prefer.

### Option A — Keychain Access (GUI)

1. Open **Keychain Access** (Spotlight → "Keychain Access").
2. Menu bar → **Keychain Access → Certificate Assistant → Create a
   Certificate…**
3. **Name:** `JC Grid Manager Dev` — must match exactly, since this is the
   string you'll put in `APPLE_SIGNING_IDENTITY` later.
4. **Identity Type:** Self Signed Root
5. **Certificate Type:** **Code Signing** — the default is "SSL"; change it.
   This is the field that matters most.
6. Optional but recommended: check **"Let me override defaults"** and bump
   the validity period (default is 1 year) to something like **10 years /
   3650 days**, so you don't have to redo this often. Step through the rest
   of the wizard with defaults; on the "Extended Key Usage Extension" screen,
   confirm **Code Signing** is checked.
7. **Keychain:** login (default). Finish.
8. Find the new certificate in Keychain Access under the **login** keychain →
   **My Certificates**. Double-click it → expand **Trust** → set **Code
   Signing** to **Always Trust** → close (enter your password if prompted).
   This avoids `codesign` verification warnings later; it doesn't need to be
   trusted by anyone but your own machine.

### Option B — scripted CLI flow (`openssl` + `security`)

Equivalent to Option A. Useful for scripting a fresh-machine setup instead of
clicking through the wizard. Run from a scratch directory — the loose
key/cert/`.p12` files aren't needed once they're imported into the keychain.

```bash
# 1. Config for a self-signed *code signing* certificate (the
#    extendedKeyUsage line is what makes it a code-signing cert, not a TLS one).
cat > codesign.cnf <<'EOF'
[req]
distinguished_name = dn
x509_extensions = codesign_ext
prompt = no

[dn]
CN = JC Grid Manager Dev

[codesign_ext]
keyUsage = critical, digitalSignature
extendedKeyUsage = critical, codeSigning
basicConstraints = critical, CA:false
EOF

# 2. Generate a private key + self-signed cert (10-year validity).
openssl req -x509 -newkey rsa:2048 -keyout dev.key -out dev.crt \
  -days 3650 -nodes -config codesign.cnf

# 3. Package into a .p12 so `security import` can install both the key and cert.
openssl pkcs12 -export -inkey dev.key -in dev.crt -out dev.p12 \
  -name "JC Grid Manager Dev" -passout pass:temp1234

# 4. Import into the login keychain. -T grants codesign access to the private
#    key so it can sign without a keychain prompt on every build.
security import dev.p12 -k ~/Library/Keychains/login.keychain-db \
  -P temp1234 -T /usr/bin/codesign

# 5. Trust it for code signing — the CLI equivalent of "Always Trust" above.
#    -d sets it as the default trust setting, scoped to your login keychain
#    (no sudo/admin needed).
security add-trusted-cert -d -r trustRoot -p codeSigning \
  -k ~/Library/Keychains/login.keychain-db dev.crt

# 6. Clean up the loose files now that the identity lives in the keychain.
rm codesign.cnf dev.key dev.crt dev.p12
```

**Gotcha:** the first `codesign` invocation that uses this key may pop a
"`codesign` wants to use your confidential information stored in ... login
keychain" dialog. Click **Always Allow** — it's one-time. If it keeps
reappearing, run this once (it needs your login keychain password, so don't
put it in a script that gets committed anywhere):
```bash
security set-key-partition-list -S apple-tool:,apple:,codesign: -s \
  -k <your-login-keychain-password> ~/Library/Keychains/login.keychain-db
```

### Verify the certificate is usable

```bash
# Should list "JC Grid Manager Dev" with a SHA-1 hash, e.g.:
#   1) ABCDEF0123... "JC Grid Manager Dev"
security find-identity -v -p codesigning

# Smoke-test signing against any harmless file, e.g. a throwaway copy of /bin/echo:
cp /bin/echo /tmp/sign-test
codesign -s "JC Grid Manager Dev" --force /tmp/sign-test
codesign -dvv /tmp/sign-test
# ^ look for a line like: Authority=JC Grid Manager Dev
rm /tmp/sign-test
```

## 2. Point the build at it via `APPLE_SIGNING_IDENTITY`

The repo never hardcodes a certificate name — each machine's certificate is
referenced through an environment variable that every dev sets locally.

1. Copy the example env file:
   ```bash
   cp .env.example .env
   ```
   `.env` is git-ignored (see `.gitignore`), so this stays local to your
   machine and is never committed.

2. `.env` should contain:
   ```
   APPLE_SIGNING_IDENTITY="JC Grid Manager Dev"
   ```
   If you named your certificate something else in step 1, put that name
   here instead — just keep it out of any committed file.

3. Load it into your shell before running Tauri commands:

   **Per session** (portable, no extra tooling required):
   ```bash
   set -a; source .env; set +a
   pnpm tauri build
   ```

   **Inline, one-off:**
   ```bash
   APPLE_SIGNING_IDENTITY="JC Grid Manager Dev" pnpm tauri build
   ```

   If you'd rather not repeat this per terminal tab, add
   `export APPLE_SIGNING_IDENTITY="JC Grid Manager Dev"` to your shell
   profile (`~/.zshrc`) instead. Either works — `.env` is just the form this
   repo standardizes on so the expected variable name is discoverable.

The Tauri CLI reads `APPLE_SIGNING_IDENTITY` at build time and uses it to
sign the `.app` it produces. Per Tauri's own docs, it "overwrites
`tauri.conf.json > bundle > macOS > signingIdentity`"
(https://v2.tauri.app/reference/environment-variables/) — so the env var
alone is sufficient and nothing in committed config needs to change.

### Important nuance: `pnpm tauri build` vs `pnpm tauri dev`

- **`pnpm tauri build`** (or **`pnpm tauri build --debug`** for a faster,
  non-optimized build) runs Tauri's full bundler, which signs the resulting
  `.app` with `APPLE_SIGNING_IDENTITY`. This is the reliable path. The `.app`
  lands at:
  ```
  src-tauri/target/debug/bundle/macos/jc-grid-manager.app     # --debug
  src-tauri/target/release/bundle/macos/jc-grid-manager.app   # release
  ```
  (the folder name matches `productName` in `tauri.conf.json`, currently
  `jc-grid-manager` — note this is *not* title-cased "JC Grid Manager.app";
  adjust the path if `productName` ever changes.) Grant Accessibility to this
  `.app` once; rebuilding with the same `APPLE_SIGNING_IDENTITY` keeps the
  same identity, so the grant survives.

- **`pnpm tauri dev`** runs `cargo run` directly and launches the raw debug
  binary (`src-tauri/target/debug/jc-grid-manager`) — it does **not** go
  through the bundler/signing step, so `APPLE_SIGNING_IDENTITY` currently has
  no effect on what `tauri dev` launches (verified by reading the Tauri v2
  CLI source: `dev` shells out to `cargo run` and never touches the
  `tauri-macos-sign`/bundler code path that `build`/`bundle` use). In
  practice, verify Accessibility-dependent behavior via `pnpm tauri build
  --debug` + launching the built `.app`, not via `pnpm tauri dev`, until
  dev-mode signing is wired in.

  A known way to close this gap later (**not done here** — it would require
  adding `src-tauri/.cargo/config.toml`, which is out of scope for this
  change, see below): add a Cargo `runner` for the macOS target that
  codesigns the freshly-built binary with `$APPLE_SIGNING_IDENTITY` before
  executing it. Cargo invokes that runner for every `cargo run`, which is
  what `tauri dev` uses under the hood, so it would cover the dev loop too.
  Leaving this as a documented follow-up rather than implementing it, since
  it touches `src-tauri/`.

## 3. Verify the grant survives a rebuild

1. `pnpm tauri build --debug`
2. Launch `src-tauri/target/debug/bundle/macos/jc-grid-manager.app`, trigger
   the Accessibility prompt (or add the app manually via System Settings →
   Privacy & Security → Accessibility), and grant it.
3. Confirm the identity:
   ```bash
   codesign -dvv src-tauri/target/debug/bundle/macos/jc-grid-manager.app
   ```
   Look for `Authority=JC Grid Manager Dev`.
4. Make a trivial code change, rebuild (`pnpm tauri build --debug` again),
   relaunch the `.app`.
5. Repeat step 4 once more.
6. If the app is still listed and checked under Accessibility both times,
   without re-adding it, the identity is stable and doing its job.

## 4. Recovery: when the identity drifts anyway

If the certificate gets deleted/regenerated under a different name, or
Accessibility otherwise gets stuck in a "looks like it should be trusted but
isn't" state, reset the grant and start clean:

```bash
tccutil reset Accessibility com.jochristianto.jcgridmanager
```

Then relaunch the app and re-grant. This is the same recovery command
referenced by the first-run onboarding flow (issue 030) for the
unsigned-app "stale grant" case in general.

## Files involved

- `.env.example` — documents the `APPLE_SIGNING_IDENTITY` variable name (committed).
- `.env` — your local certificate name (git-ignored, not committed).
- This doc.

No changes were made to `src-tauri/tauri.conf.json` or anything else under
`src-tauri/` — the env var is sufficient on its own (see above), and that
tree was mid-edit by other work at the time this was written.
