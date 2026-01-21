# Simple Rust Shell

A minimal POSIX-like shell written in Rust. This project was developed as a solution to the "Build Your Own Shell" challenge on CodeCrafters.

## Description

This is a command-line interpreter that provides a basic interactive prompt. It can parse user commands, execute external programs found in the system's `PATH`, and handle several built-in commands.

One notable feature is the multi-threaded approach to discovering executable files, which is used to populate the list of available commands at startup.

## Features

- Interactive prompt using `$` as the prefix.
- Execution of external commands (e.g., `ls`, `grep`, `cat`).
- Built-in commands:
    - `exit`: Terminates the shell.
    - `echo`: Prints arguments to the standard output.
    - `pwd`: Displays the current working directory.
    - `cd`: Changes the current working directory. Supports `~` for the home directory.
    - `type`: Indicates whether a command is a built-in or an external executable.
- Multi-threaded search for executables in the directories listed in the `PATH` environment variable.

### Prerequisites

- Rust and Cargo must be installed. You can install them from [rust-lang.org](https://www.rust-lang.org/tools/install).

### Building

1.  Clone the repository.
2.  Build the project:
    ```sh
    cargo build --release
    ```

### Running

You can run the shell in two ways:

1.  Using Cargo:
    ```sh
    cargo run
    ```
2.  By running the compiled binary directly:
    ```sh
    ./target/release/codecrafters-shell
    ```

## Usage

Once the shell is running, you can type commands at the prompt.

**Executing an external command:**
```sh
$ ls -l
```

**Using a built-in command:**
```sh
$ echo Hello, World!
Hello, World!
```

**Changing directories:**
```sh
$ pwd
/home/user/codecrafters-shell-rust
$ cd /tmp
$ pwd
/tmp
$ cd ~
$ pwd
/home/user
```

**Checking a command's type:**
```sh
$ type echo
echo is a shell builtin
$ type ls
ls is /bin/ls
$ type unknown_command
unknown_command: not found
```

**Exiting the shell:**
```sh
$ exit
```
