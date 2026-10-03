# Production Native Signing and Notarization

## Purpose

The normal Desktop Artifacts workflow remains an unsigned internal qualification builder. Production-native signing is a separate trust boundary and must never be inferred from Sigstore/SLSA provenance alone.

## Windows production contract

The signing provider and private-key custody are external deployment choices. No PFX, private key, provider credential, or certificate secret belongs in this repository.

A production Windows release must satisfy all of the following before publication:

- Authenticode file digest algorithm: SHA-256.
- RFC 3161 timestamping is required.
- Timestamp digest algorithm: SHA-256.
- The certificate must be valid for Code Signing EKU `1.3.6.1.5.5.7.3.3`.
- The final installer and executable must report a valid Authenticode signature.
- SignTool verification must pass using the Authenticode policy and all embedded signatures.
- Evidence must show a valid RFC 3161 timestamp.

The repository deliberately does not choose PFX, hardware token, Azure Artifact Signing, Key Vault, or another provider. Provider credentials are supplied only at the external production-signing boundary.

## macOS production contract

Internal qualification uses ad-hoc signing. Production distribution must use the production configuration overlay:

`apps/kaspa-gateway-desktop/src-tauri/tauri.macos.production.conf.json`

That overlay clears the ad-hoc identity and enables Hardened Runtime. The actual Developer ID Application identity is supplied through `APPLE_SIGNING_IDENTITY`.

Notarization credentials must be supplied externally using exactly one supported Tauri credential set:

- App Store Connect API: `APPLE_API_ISSUER`, `APPLE_API_KEY`, `APPLE_API_KEY_PATH`; or
- Apple ID: `APPLE_ID`, `APPLE_PASSWORD`, `APPLE_TEAM_ID`.

A production macOS release is incomplete until all of these are proven on the exact artifacts:

- Developer ID Application signing.
- Hardened Runtime enabled.
- Secure timestamp present.
- Apple notarization accepted through `notarytool`.
- Notarization ticket stapled and validated.
- Gatekeeper assessment passes.
- Existing architecture, smoke, SBOM, checksum and provenance gates remain green.

## Trust separation

Build provenance, native platform signing, and repository governance are independent controls. None may be substituted for another.

The local readiness gate only proves that repository policy and production configuration are prepared. It does not claim that production credentials exist, that Apple notarization has run, that Windows artifacts have been signed, or that remote governance has been enforced.

Production trust can be closed only after exact-head remote CI, artifact attestations, native signing/notarization evidence, and required repository governance are all independently verified.
