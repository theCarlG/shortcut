# Shortcut
Shortcut is a **Work In Progress** & **Proof Of Concept** application with convient shortcuts for some settings that I change often on my SteamDeck.

Shortcut consists of a GUI application running as the **deck** user in the foreground and a gRPC server running in the background as **root**.

Since this is a test project I wont spend any time to make the shortcuts configurable.

## Screenshot
![Screenshot](https://github.com/theCarlG/shortcut/blob/main/.github/images/app-gui.png)

## Installing
Run the following command on your Steam Deck
```bash
curl https://raw.githubusercontent.com/theCarlG/shortcut/main/scripts/install.sh | bash
```
You have to add **Shortcut** as a Non Steam Game in Steam when the installation is done

## Why?

I wanted to try and make a GUI application for my SteamDeck while learning Rust 🦀 and [egui](https://github.com/emilk/egui)
