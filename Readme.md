[![discord](https://img.shields.io/badge/Discord-blue?style=for-the-badge)](https://discord.gg/zTvhS7eYuQ)
# Streamduck
![streamducklogo_cut](https://user-images.githubusercontent.com/12719947/151142599-07620c87-3b51-4a65-b956-4a5902f2f52c.png)
<br>
Software for managing Stream Deck devices with folders and actions

*Project that will perhaps be better than streamdeck-ui*

### Currently still in heavy development, and is not usable

# Features
## Currently supported:
* **Linux compatible**: Works on Linux without having to code
* **Managing multiple streamdeck devices**: Able to control multiple streamdeck devices with each having their own separate configurations.
* **Flexible button display**: Buttons can be configured to have one of the following backgrounds: (solid color, horizontal and vertical gradients, images), and any amount of text objects with extensive text rendering settings
* **Folders**: Buttons can be structured in any desirable folder structure
* **Plugin support**: The software can be extended with any amount of plugins, with plugins having access to all core features of the project. In fact, folder support was made with same API as plugins use.
* **Auto-(re)connect**: Will automatically attempt to establish connection with previously added devices
## Planned features:
* **Lua support for plugins** - for simple plugins to be made that utilize streamduck API
* **Support for animated images**
* **Support for plugins to render custom images** - will allow much greater flexibility for plugins, you could potentially run games on streamdeck screen
* **Windows support** for those who would prefer using open source software on Windows
* **Button animation system** with flexibility of having plugins define custom animations
* **Built-in OBS Websocket integration**
* **NodeJS native module**
* **Electron-based GUI** (will have equal functionality with cli version) (alternative GUI applications can be made)

# Structure of the Project
## streamduck-core
Simplification of rendering and streamdeck management for use with other modules
## streamduck-daemon
Service that handles the core, plugins and provides a many types of sockets for clients to interact with the core<br>
Communication sockets that are currently planned are as following: Unix Domain Socket, Windows Named Pipes, Websocket
## streamdeck-client
Library that simplifies communication with the streamduck-daemon, and also allows one to make custom UI clients for the software
## streamdeck-cli
Command-line tool to interact with the daemon
