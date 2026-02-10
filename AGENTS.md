

 This project adds Discord Rich Presence support to the Helix editor by running
 a small, standalone Language Server written in Rust. The server uses the tower-lsp crate to communicate with Helix through the Language Server Protocol and listens to standard editor events such as opening, changing, and closing documents.
 
 Based on these LSP events, the server infers the current editing context, including the active file, the programming language, and the workspace. It then publishes this information to Discord using the discord-presence crate, which communicates with the locally running Discord client via IPC.
 
 The language server does not implement any language features like diagnostics, code completion, or formatting. It exists solely as an integration layer between Helix and Discord and remains active for the duration of the editor session. Because Helix does not support extensions, using an LSP server allows this integration without modifying Helix itself or replacing its built-in language servers.

