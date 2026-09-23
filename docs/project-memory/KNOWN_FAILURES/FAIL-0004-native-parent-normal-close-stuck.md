# FAIL-0004: Native parent remains after authorized normal close

- Status: BLOCKED
- Date: 2026-09-23
- Category: KNOWN FAILURE
- Affected scope: Windows Server native desktop close path for the v2.1.0 qualification parent; runtime workers are not affected.

## Evidence
Exact parent PID 4404, started from `C:\KGW-AUD\v210-target\debug\kaspa-gateway-desktop.exe` with artifact SHA-256 `39A7E1D923414677F8510DCEC2B6EACA4F01317D7A1E1868E3F2821BCA12F3A2`, remained running after `CloseMainWindow` returned true and after Tauri `getCurrentWindow().close()` was scheduled. Independent `shutdown-all` returned `stopped=0`; Mainnet/Testnet10 Node/Bridge owner statuses were stopped, no child KGW runtime workers remained, and task runtime ports were free. Tauri `Window.destroy()` is blocked by `core:window:allow-destroy` capability policy. No OS force-kill was used.

## Root Cause
NOT YET CONFIRMED. The planned no-file-trace A/B comparison cannot start while the exact parent is still present because single-parent policy forbids launching a second desktop parent. File-trace finalization is therefore only a hypothesis, not a confirmed cause.

## Fix / Decision
Fail closed. Preserve the exact idle parent and all evidence. Do not weaken Tauri capabilities or use OS force termination merely to complete the experiment. Defer the no-file-trace A/B until the parent exits naturally or a separately authorized safe termination capability exists.

## Verification
- Four planned native runtime cases remain VERIFIED_SUCCESS and are not invalidated by the idle parent.
- Runtime workers after qualification: zero.
- Task runtime ports after qualification: free.
- Production lease SHA-256 remained unchanged at `6B173A290E47E729C38D408DB3227638D03E8ADAB18C94F921DEB656E163459C`.
- Normal close attempts did not terminate the parent.
- Safe destroy is capability-blocked.
- Forced termination: NOT AUTHORIZED / NOT USED.

## Regression Protection
The operation journal binds the parent PID/start/artifact identity and explicitly forbids duplicate parent launch, force-kill, and false success claims. Continuity state carries this blocker forward until a safe retest is possible.

## Remaining Risk
One idle qualification desktop parent remains on Server and prevents a clean single-parent close-root-cause A/B. The root cause is unresolved. No active Kaspa runtime worker or task listener is associated with it.

## NEXT ACTION
After a separately authorized safe termination path becomes available or PID 4404 exits naturally, verify no KGW runtime workers/listeners remain and run exactly one no-file-trace A/B with the same artifact and no runtime start. Until then, continue only repository-local work that does not require a second parent.

## DO NOT REPEAT
Do not repeat `CloseMainWindow`, Tauri normal close, or capability-blocked destroy attempts without a changed close capability/environment. Do not force-kill PID 4404 or start a second desktop parent.
