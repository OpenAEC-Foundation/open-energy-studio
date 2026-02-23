# Open Energy Studio - Development Guidelines

**READ THIS FILE BEFORE MAKING ANY CHANGES.**

## Code Structure Reference

For detailed documentation of the codebase structure, see **[CodeStructure.md](./CodeStructure.md)**.

**IMPORTANT:** When adding, removing, or modifying files/folders, update `CodeStructure.md` to reflect the changes.

---

## Development Commands

### Run Tauri Dev Server
```bash
cd "D:\Repos\(Impertio)\open-energy-studio" && npm run tauri dev
```
Run this in background mode with a 10-minute timeout.

## Versioning

The app uses **CalVer** format: `YEAR.MONTH.BUILD`
- `YEAR` — full calendar year (e.g. 2026)
- `MONTH` — month number, no leading zero (e.g. 2 for February)
- `BUILD` — build number, starts at 0 each month, incremented on each push

Example: `2026.2.0` → `2026.2.1` → ... → `2026.3.0`

Version must be kept in sync across these files (use `npm run bump <version>`):
- `package.json`
- `src-tauri/tauri.conf.json`
- `src-tauri/Cargo.toml`

The version is displayed in the title bar via `import { version } from '../../../package.json'`.

---

## General
- when I ask to do something, alwayse first tell me what you have understand form my command (in short) and after my confirmation, then continue.
- When you impliment something, think that if that modifying are required based on the changes for the following items:
  - APIs
  - Keyboard Shortcuts in the File ribbon
  - Saving/Loaing file
  - Export to ifc file


## Github commit
- Remove all comments that reference to AutoCAD, Revit, Autodesk softwares or Microsoft or any third parties that are not opensource
- make sure it does not generates error on github action.
- Create a commit comment by comparing the current files to the commited changes.
- Do not include these in the commit comment:
  - the Claude and Anthropic stuff
  - Co-Authored-By: Claude Opus 4.6 or something like this
  - about that the 3rd parties names are removed
- Show the commit comment to user and ask for approval.
- when wanted to commit and push, increase the build number in the version everywhere.
- For every push, run github action to create a new installers. wait to the end make sure they are created successfully.
- also publish the a new draft release

## Github issues
- when replying or closeing an issue, do not mention 3rd parties softwares
- before possing something alwayse ask for approval