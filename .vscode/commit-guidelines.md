# Conventional Commits Usage Guide

> 📘 This document includes material from the [Conventional Commits v1.0.0 Specification](https://www.conventionalcommits.org/en/v1.0.0/), licensed under [Creative Commons Attribution 3.0 International (CC BY 3.0)](https://creativecommons.org/licenses/by/3.0/).  
> Copyright © The Conventional Commits Specification Authors.

---

## ✍️ Commit Message Format

```text

<type>(<scope>)\<optional !>: <description>

\[optional body]

\[optional footer(s)]

````

- The message must start with a **type** (`feat`, `fix`, `docs`, etc.).
- A **scope** may be included in parentheses.
- If the commit includes a breaking change, **`!`** must be added before the colon.
- A **description** must follow after a colon and a space.

---

## ✅ Examples

### Basic Feature and Fix

```text
feat(login): add support for OAuth2

fix(parser): handle null inputs correctly
````

### With Body

```text
feat(profile): allow avatar image uploads

This adds support for uploading avatar images using multipart/form-data.
Thumbnails are generated automatically using sharp.
```

### With Footer

```text
fix(auth): reject expired tokens

Tokens are now checked for expiration during validation.

Closes: #42
```

### Breaking Change (in header)

```text
refactor(api)!: drop support for legacy v1 endpoints

BREAKING CHANGE: API clients must now use the v2 endpoints.
```

### Breaking Change (footer only)

```text
chore(deps): update express to v5

BREAKING CHANGE: Express 5 no longer supports middleware chaining via `next('route')`.
```

---

## 📌 Common commit types and gitmoji

Use these types and emojis for commit messages. Each type should be used according to the change's intent.

| Emoji | Type         | Description                        |
| :---- | :----------- | :--------------------------------- |
| ✨    | feat         | Add a new feature                  |
| 🐛    | fix          | Fix a bug                          |
| 📝    | docs         | Add or update documentation        |
| 🎨    | style        | Improve code style or formatting   |
| ♻️    | refactor     | Refactor code (no behavior change) |
| ⚡️    | perf         | Improve performance                |
| ✅    | test         | Add or update tests                |
| 🔧    | chore        | Build process or tool changes      |
| 🚧    | wip          | Work in progress                   |
| 🔥    | remove       | Remove code or files               |
| ⬆️    | upgrade      | Upgrade dependencies               |
| ⬇️    | downgrade    | Downgrade dependencies             |
| ⏪️    | revert       | Revert changes                     |
| 🗑️    | deprecate    | Mark code as deprecated            |
| 💚    | ci           | CI related changes                 |
| 🐳    | docker       | Docker related changes             |
| 🧪    | test-fail    | Add a failing test                 |
| 🛂    | auth         | Authentication/authorization       |
| 🌐    | i18n         | Internationalization/localization  |
| 💄    | ui           | UI or visual changes               |

You may define and use additional types if needed.

---

## 🔒 Enforcement (Optional)

To enforce Conventional Commits automatically, consider using:

- [`commitlint`](https://github.com/conventional-changelog/commitlint)
- [`husky`](https://typicode.github.io/husky)

These tools help prevent invalid commit messages during local development or CI workflows.

---

## 🔗 References

- [Official Specification](https://www.conventionalcommits.org/en/v1.0.0/)
- [RFC 2119 Keywords](https://www.rfc-editor.org/rfc/rfc2119)
- [CC BY 4.0 License](https://creativecommons.org/licenses/by/4.0/)
