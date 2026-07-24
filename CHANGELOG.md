# Changelog

## [Unreleased]


### Bug Fixes

* **windows:** default shortcuts now use a Ctrl+Alt+Shift base so they no longer collide with AltGr, Intel screen-rotation hotkeys, or third-party Ctrl+Alt binds (e.g. a "new desktop" grab on Ctrl+Alt+D); Next/Previous Display and Maximize Height ship unbound on Windows (tray-reachable, rebindable) to avoid five-key combos

## [0.2.0](https://github.com/jochristianto/jc-grid-manager/compare/v0.1.0...v0.2.0) (2026-07-08)


### Features

* **core:** add action registry, default binds, and shortcut dispatcher ([19ba963](https://github.com/jochristianto/jc-grid-manager/commit/19ba963a9f220d41ff93bea61a617180681a4c42))
* **core:** add Almost Maximize action (configurable, default 90%) ([4c38492](https://github.com/jochristianto/jc-grid-manager/commit/4c3849291051d595a8c1629ae458f68b1825c9f7))
* **core:** add Center action (⌃⌥C) ([ad21696](https://github.com/jochristianto/jc-grid-manager/commit/ad21696221489a4ab3d9417d1c76db08eba179b7))
* **core:** add corner quarter actions (⌃⌥U/I/J/K) ([bd9e45f](https://github.com/jochristianto/jc-grid-manager/commit/bd9e45fd497dbde68cc936424498edc66d5c7f75))
* **core:** add First/Center/Last Third actions (⌃⌥D/F/G) ([3411404](https://github.com/jochristianto/jc-grid-manager/commit/34114046076870b7d76af2a662509a4a81ef1d59))
* **core:** add First/Last Two Thirds actions (⌃⌥E/T) ([0457832](https://github.com/jochristianto/jc-grid-manager/commit/0457832bedd79f247bfb4040688400f62b811d37))
* **core:** add Fourths actions ([d637efe](https://github.com/jochristianto/jc-grid-manager/commit/d637efeb58d5dd842b789a3c05bb0a7822fe24ae))
* **core:** add geometry table + Center Half action ([1fdb965](https://github.com/jochristianto/jc-grid-manager/commit/1fdb965d4fda1ad3b3c784f15eee903d125d6ffb))
* **core:** add Maximize action (⌃⌥↩) ([5bc572f](https://github.com/jochristianto/jc-grid-manager/commit/5bc572f839a8eba0e09f6d9232cec97e47dab88d))
* **core:** add Maximize Height action (⌃⌥⇧↑) ([7242a99](https://github.com/jochristianto/jc-grid-manager/commit/7242a998a1cae95f489813d9b7cab09e1c0bbdf3))
* **core:** add Move to Edge actions (left/right/up/down) ([b94c801](https://github.com/jochristianto/jc-grid-manager/commit/b94c801618725d05fb87ebedb00898e44aea592e))
* **core:** add Next/Previous Display actions (⌃⌥⌘←/→) ([900cd17](https://github.com/jochristianto/jc-grid-manager/commit/900cd17a80d314dc1f61b6ea25849845f9e3ff36))
* **core:** add optional launch-at-login via autostart plugin ([99af47a](https://github.com/jochristianto/jc-grid-manager/commit/99af47a6bfb517b8db1e75158ebd7621bdefafbe))
* **core:** add per-app ignore list ([73bb652](https://github.com/jochristianto/jc-grid-manager/commit/73bb6526eea3cdb9cbff29dfa594aaace50d804f))
* **core:** add Restore action (⌃⌥⌫) ([12ba114](https://github.com/jochristianto/jc-grid-manager/commit/12ba114e675d2ef89418390eab768b8e3e37585f))
* **core:** add Sixths actions (3×2 grid) ([fd36050](https://github.com/jochristianto/jc-grid-manager/commit/fd36050f3e42c04433286d14de6ec27430ba68eb))
* **core:** add Smaller/Larger actions (⌃⌥- / ⌃⌥=) ([3292d16](https://github.com/jochristianto/jc-grid-manager/commit/3292d16cfdcdfa17b5a8a8c36119b2847ab46329))
* **core:** add snap state machine with size cycling and restore baseline ([8d4398b](https://github.com/jochristianto/jc-grid-manager/commit/8d4398be2fbeb4f64735c30ff613577a10d823d4))
* **core:** persist local config with validated live rebinding ([3fe6800](https://github.com/jochristianto/jc-grid-manager/commit/3fe6800ffed3039afe00775c193842665bf9641b))
* **core:** select target display by largest overlap ([888c02b](https://github.com/jochristianto/jc-grid-manager/commit/888c02b7771a595ad259d2b408ed26e89b598e2a))
* **core:** soft beep when an action can do nothing ([7ca90c8](https://github.com/jochristianto/jc-grid-manager/commit/7ca90c8f4902021b2381bfa7593f0abc0f5ea9fc))
* **tray:** full menu mirroring the Rectangle feature set ([5b71af5](https://github.com/jochristianto/jc-grid-manager/commit/5b71af5bae1ca78de5e9b4e559123deeeaec8cd6))
* **ui:** add macOS Accessibility onboarding (three states) ([dfe45ac](https://github.com/jochristianto/jc-grid-manager/commit/dfe45aca9ad3dfffc987901175adc324529d9d4e))
* **ui:** add Settings window shell with tabbed navigation ([13eaf1a](https://github.com/jochristianto/jc-grid-manager/commit/13eaf1a56085f7996bca51fa12456b218b7173b5))
* **ui:** add shortcut editor with key recording and conflict warnings ([b1d1ab1](https://github.com/jochristianto/jc-grid-manager/commit/b1d1ab1e63b159bb1af02442ab1f31b48bd508fe))
* **windows:** beep on elevated windows and warn on Ctrl+Alt conflicts ([ca1da9d](https://github.com/jochristianto/jc-grid-manager/commit/ca1da9de2d22a070c54d0bf686f0550303aa1ac5))
* **windows:** implement the Platform trait via Win32 ([fdc361b](https://github.com/jochristianto/jc-grid-manager/commit/fdc361b7cf4efb9bb32df5344f8d91987a79005a))


### Bug Fixes

* **packaging:** replace placeholder crate metadata ([deeb9b2](https://github.com/jochristianto/jc-grid-manager/commit/deeb9b2b60e2c7af7280d4a4e158dac16640f25c))
* **packaging:** replace placeholder crate metadata ([a0afc65](https://github.com/jochristianto/jc-grid-manager/commit/a0afc65b9e809ad144e93fb1336c10afa9c25e49))
* **windows:** declare per-monitor-DPI-v2 for mixed-DPI multi-monitor snaps ([62d4389](https://github.com/jochristianto/jc-grid-manager/commit/62d43896b4a1c03c28721f65b828f227bf2676a4))
