# Streamduck
![streamducklogo_cut](https://user-images.githubusercontent.com/12719947/151142599-07620c87-3b51-4a65-b956-4a5902f2f52c.png)
<br>
Open Source and Cross Platform software to manage macro devices like Elgato Stream Deck

### Currently in heavy development, groundwork is being laid!

#### [docs.streamduck.org](https://docs.streamduck.org/) will be rewritten once the project is ready to be used
#### [Learn why I switched to C# for this project](why-dotnet.md)
#### If you're looking for Rust version's code, [here it is](https://github.com/streamduck-org/streamduck/tree/old-master)

## Project Structure

### Streamduck
Main functionality, the daemon executable and plugin loading functionality

### StreamduckShared
Definitions that are shared between the app and the plugin

### StreamduckStreamDeck
Plugin that adds Stream Deck device support into Streamduck

### ElgatoStreamDeck
Library that is used by StreamduckStreamDeck to interact with Stream Deck devices