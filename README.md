## About

This is a [Zellij](https://github.com/zellij-org/zellij) plugin that displays the active keybinds and their corresponding actions.

![image](https://github.com/user-attachments/assets/0b83b48d-2495-49c8-8bb9-0ac05961a58f)

## Development

```bash
zellij action new-tab --layout ./plugin-dev-workspace.kdl
```
Test:
* `cargo wasi test --lib`
* `cargo insta test --target x86_64-unknown-linux-gnu --lib`
