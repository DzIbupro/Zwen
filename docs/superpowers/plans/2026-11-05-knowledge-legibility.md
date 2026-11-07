# Knowledge toolbar and text legibility

Direct implementation of explicit items6/8: retain neutral Soft UI and current local fonts. Raise muted text contrast in both themes, give auxiliary labels moderate weight; keep disabled states visibly disabled. Knowledge functions form a labeled raised tool area; topic chips are flat pill filters with their own label. Organize title/return and mixed controls align on the vertical center; wrapping remains supported. No new feature or data/model behavior.

- [x] Modify app.css and KnowledgePanel styles/markup only; preserve selectors/handlers.
- [x] Actual-component browser fixture: light/dark, narrow layout, organize toolbar alignment. Run frontend tests/check/build and existing isolated knowledge regression after rebuilding smoke.
- [x] Review, ordinary build, integrate/deploy, record remaining items.
