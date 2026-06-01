#!/usr/bin/env python3
"""Inject release signing config into the Tauri-generated Android Gradle build.

Run AFTER `tauri android init` and BEFORE `tauri android build`.

This patches `src-tauri/gen/android/app/build.gradle.kts` so the release APK is
signed with the keystore described by `keystore.properties`, which this script
also writes from environment variables.

It is **self-validating**: if any of the three required edits cannot be applied,
the script exits non-zero so CI fails loudly instead of silently producing an
unsigned release APK. It is also idempotent — running twice is a no-op.

Required environment variables:
  CUCKOO_KEYSTORE_PATH   absolute path to the decoded .jks (default /tmp/cuckoo.jks)
  ANDROID_KEY_ALIAS      key alias
  ANDROID_KEY_PASSWORD   key password
  ANDROID_STORE_PASSWORD keystore password
"""
import os
import re
import sys

GRADLE = "src-tauri/gen/android/app/build.gradle.kts"
PROPS = "src-tauri/gen/android/keystore.properties"


def die(msg: str) -> "None":
    print(f"ERROR: {msg}", file=sys.stderr)
    sys.exit(1)


def find_block_end(text: str, open_brace_idx: int) -> int:
    """Return index of the `}` that closes the block whose `{` is at open_brace_idx."""
    depth = 0
    i = open_brace_idx
    while i < len(text):
        c = text[i]
        if c == "{":
            depth += 1
        elif c == "}":
            depth -= 1
            if depth == 0:
                return i
        i += 1
    die("unbalanced braces while scanning Gradle block")


def main() -> "None":
    keystore_path = os.environ.get("CUCKOO_KEYSTORE_PATH", "/tmp/cuckoo.jks")
    alias = os.environ.get("ANDROID_KEY_ALIAS", "")
    key_pw = os.environ.get("ANDROID_KEY_PASSWORD", "")
    store_pw = os.environ.get("ANDROID_STORE_PASSWORD", "")

    if not (alias and key_pw and store_pw):
        die("ANDROID_KEY_ALIAS / ANDROID_KEY_PASSWORD / ANDROID_STORE_PASSWORD must all be set")
    if not os.path.isfile(keystore_path):
        die(f"keystore not found at {keystore_path}")
    if not os.path.isfile(GRADLE):
        die(f"{GRADLE} not found — run `tauri android init` first")

    # 1) Write keystore.properties (Gradle root is gen/android, so this sits there).
    with open(PROPS, "w") as f:
        f.write(
            f"storeFile={keystore_path}\n"
            f"storePassword={store_pw}\n"
            f"keyAlias={alias}\n"
            f"keyPassword={key_pw}\n"
        )

    with open(GRADLE) as f:
        src = f.read()

    if 'signingConfig = signingConfigs.getByName("release")' in src:
        print("Signing config already present — skipping Gradle patch.")
        return

    # 2) Properties loader at top level, immediately before `android {`.
    loader = (
        'val keystorePropertiesFile = rootProject.file("keystore.properties")\n'
        "val keystoreProperties = java.util.Properties().apply {\n"
        "    if (keystorePropertiesFile.exists()) keystorePropertiesFile.inputStream().use { load(it) }\n"
        "}\n\n"
    )
    m = re.search(r"(?m)^android\s*\{", src)
    if not m:
        die("`android {` block not found")
    src = src[: m.start()] + loader + src[m.start():]

    # 3) signingConfigs block right after `android {`.
    m = re.search(r"(?m)^android\s*\{", src)
    signing_block = (
        "\n    signingConfigs {\n"
        '        create("release") {\n'
        '            (keystoreProperties["keyAlias"] as String?)?.let { keyAlias = it }\n'
        '            (keystoreProperties["keyPassword"] as String?)?.let { keyPassword = it }\n'
        '            (keystoreProperties["storePassword"] as String?)?.let { storePassword = it }\n'
        '            (keystoreProperties["storeFile"] as String?)?.let { storeFile = file(it) }\n'
        "        }\n"
        "    }\n"
    )
    src = src[: m.end()] + signing_block + src[m.end():]

    # 4) Add `signingConfig = ...` inside getByName("release") { ... } via brace matching.
    key = 'getByName("release")'
    idx = src.find(key)
    if idx == -1:
        die('`getByName("release")` not found in buildTypes')
    brace = src.find("{", idx)
    if brace == -1:
        die("opening brace for release buildType not found")
    end = find_block_end(src, brace)
    inject = '            signingConfig = signingConfigs.getByName("release")\n        '
    src = src[:end] + inject + src[end:]

    with open(GRADLE, "w") as f:
        f.write(src)

    # 5) Validate — fail loudly if either edit didn't land.
    with open(GRADLE) as f:
        out = f.read()
    if "signingConfigs {" not in out:
        die("validation failed: signingConfigs block missing after patch")
    if 'signingConfig = signingConfigs.getByName("release")' not in out:
        die("validation failed: release buildType not wired to signing config")

    print("✅ Release signing config injected and verified.")


if __name__ == "__main__":
    main()
