# Remove demo and format preview implementation plan

> Execute inline with executing-plans; scope is explicitly requested item2, no additional visual direction.

- [ ] Remove production DemoStream/sample/preview mode, menu entries and Capsule onDemo; keep real model rendering, stop, restoration and drafts.
- [ ] Regression verifies both surfaces lack demo/format-preview controls. Migrate native capsule geometry to local synthetic HTTP stream; real reply rendering is checked by native chat tests. Foundation draft lifecycle stops depending on format preview.
- [ ] Frontend tests/check/build, native chat/capsule/foundation/actions regression with temporary storage only; no real models. Review before integration, normal build without smoke hooks; atomic install with existing process protected.
