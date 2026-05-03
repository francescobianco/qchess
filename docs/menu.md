# Menu

The top menu bar contains these entries, in this order:

```text
Database     Games     Moves     Engine     Options     ?
```

Do not show an `Exit` entry in the top menu bar.

Do not show function-key labels or `=[key]` suffixes in the top menu bar.

The interface is multilingual. The UI language must be configurable.

## Engine Menu

The Engine menu opens as a dropdown below the `Engine` menu-bar entry.

The dropdown contains:

1. `Add New Engine`.
2. A separator.
3. The list of registered engines.

One registered engine may be marked as the current active engine. The active engine is shown with an active flag.

Selecting an existing engine opens an edit form. The edit form uses the same layout as the create-new-engine form.

The form fields are:

- Name.
- Command.
- Arguments.

The form buttons are:

- Delete.
- Cancel.
- Save.
- Save and Use.

`Save and Use` saves the engine and marks it as the active engine.

