[![discord](https://img.shields.io/badge/Discord-blue?style=for-the-badge)](https://discord.gg/zTvhS7eYuQ)
# Streamduck
![streamducklogo_cut](https://user-images.githubusercontent.com/12719947/151142599-07620c87-3b51-4a65-b956-4a5902f2f52c.png)
<br>
Software for managing Stream Deck devices with folders and actions

*Project that will perhaps be better than streamdeck-ui*

🛈 Currently in heavy development, and is not user-ready

## Features

### Currently supported

* **Cross Platform**: Works on Windows and on Linux
* **Managing multiple Stream Deck devices**: Able to control multiple Stream Deck devices with each having their own separate configurations.
* **Flexible button display**: Buttons can be configured to have one of the following backgrounds: (solid color, horizontal and vertical gradients, images), and any amount of text objects with extensive text rendering settings
* **Support for animated images**: GIF and APNG images are supported with rendering exactly as described in the format (no more slow-mo gifs like in original software)
* **Folders**: Buttons can be structured in any desirable folder structure
* **Plugin support**: The software can be extended with any amount of plugins, with plugins having access to all core features of the project. In fact, folder support was made with same API as plugins use.
* **Support for plugins to render custom images**: Plugins can add things to buttons, they can even define their own renderers, allowing for low-level access, you could potentially run games on Stream Deck
* **Auto-(re)connect**: Will automatically attempt to establish connection with previously added devices
* **Import/Export Configuration**: Allows you to import and export device configurations, which include images that were uploaded into the software. Making backups of configs as easy as keeping the exported file, and just importing it later when needed.

### Planned features
* **Lua support for plugins** - for simple plugins to be made that utilize streamduck API
* **OBS Websocket plugin**
* **Electron-based GUI** (will have equal functionality with CLI version) (alternative GUI applications can be made)

### Maybe in the future
* **Button animation system** with flexibility of having plugins define custom animations
* **Plugin store** for easy way to browse and install plugins
* **Support for official SDK plugins on Windows**
## NodeJS module
Client for interacting with Streamduck daemon on NodeJS<br>
[Repository](https://github.com/TheJebForge/streamduck-node-client)

## Project structure

<dl>
 <dt><tt>streamduck-core</tt></dt>
 <dd>Simplification of rendering and Stream Deck management for use with other modules</dd>
 <dt><tt>streamduck-daemon</tt></dt>
 <dd>Service that handles the core, plugins and provides a many types of sockets for clients to interact with the core.<br>
  Communication sockets that are currently planned are as following: Unix Domain Socket, Windows Named Pipes, Websocket.</dd>
 <dt><tt>streamduck-client</tt></dt>
 <dd>Library that simplifies communication with the `streamduck-daemon`, and also allows one to make custom UI clients for the software.</dd>
 <dt><tt>streamduck-cli</tt></dt>
 <dd>Command-line tool to interact with the daemon (`streamduck-daemon`).</dd>
</dl>
