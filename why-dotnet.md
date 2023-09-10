# Why I switched from Rust to C#

### Some behind the scenes first
A lot of things happened from my last commit on the Streamduck repo master, I've decided to rewrite the software
for several reasons:

I didn't like how the current available version of Streamduck could only work with Elgato
Stream Decks, I wanted to broaden the software into supporting any device that a plugin could bring.
But with the current code base it basically meant I have to rewrite it, since the code was intertwined with the
Stream Deck library...

So I started the first rewrite of the software where the only change was a more generic interface through which
plugins could implement drivers for any kind of device. But then the next problem happened...

Rust does not have a stable ABI, the plugin implementation I've done is incredibly naive and it would had failed
in many spectacular ways, since Rust compiler doesn't even guarantee that it's gonna do the same binary between
compilations on the same compiler version.

So second rewrite was focused to trying to make plugin support using C ABI, but using FFI like this brings a lot
of other problems, like the entire interface between main application and plugins being made up out of unsafe code.
I'd have to express my entire API in C ABI and make sure to not corrupt my memory at the same time, since memory 
allocated on plugin should be deallocated by the plugin...

For last half a year I was trying to figure out how to even get there with the software design, and even if I did
manage to get there, I'd have to make a lot of macros to make plugin development easy enough and every plugin would
have to compile to multiple platforms, and distribute those binaries separately..

It's basically just a huge mess I didn't want to handle. So it brings us back to the topic:

### So why did I chose C#?

Dotnet pretty natively supports running applications with plugins, making designing a plugin interface very trivial
(they even have an [official guide](https://learn.microsoft.com/en-us/dotnet/core/tutorials/creating-app-with-plugin-support) on this!).

Binaries that dotnet compiles are cross platform, the .dll file of a plugin can be used on Windows, Linux and OSX without
additional compilations or complications (unless your plugin uses native libraries, you'd have to compile those to platforms
you'd want to support).

Dotnet 7 is also very fast, it's incredibly optimized for being a semi-interpreted language. I've rewrite my Stream Deck
library to C# and C# version managed to squeeze some extra frames out of my Stream Deck Plus while flipping between 2
images with the library just writing already encoded JPEGs to the HID Device.

Dotnet being faster might have been due to rust version allocating and deallocating some memory on each write, while C#
version uses a buffer that it allocated once, and then some several small allocations of byte[] on every write. But it
still proved to me that C# can be as fast as Rust (at least in my use case).

I'm also still more familiar with OOP and how to design software around it, new features of C# make it a breeze as well.
Like all the null-coalescing operators, and the Nullable feature where every type won't be null unless you suffix it with "?".
Event dispatchers are also amazing, I don't have to make my own solution for that.

Plugin development experience is going to be much simpler on C# though, since you won't have to deal with Rust's quirks
like the borrow checker... On C#, all you have to do is define a class that implements Plugin class and it will 
automatically be loaded by Streamduck along with anything you specify in the Plugin properties, see StreamduckStreamDeck
plugin for an example.

### Not everything is perfect however...
C# does have some pitfalls, like the difference between reference types and value types and C# always copying data unless
the data is readonly, or you're using a Span to pass it instead. 

Exceptions are also not explicitly listed like they are in Java, making error handling complicated, since you need to 
check source of the method you're calling and see if it throws anything in there, thankfully at least for Standard library, 
all exceptions that you should be concerned about are included into the doc comments.

Resource usage of dotnet applications is also bigger most of the time compared to Rust, but to be honest, I'll take that
instead of having to deal with Rust's FFI...

### Conclusion
Despite some of the cons that dotnet has, I'll be able to actually finish this project in near future, instead of being
stuck on all the difficulties that Rust has doing this kind of extensible project...

The idea of this project is to be the foundation and infrastructure for using all kinds of macro devices to execute any
kind of function, all implemented using plugins.

I personally will be developing plugins for OBS Studio, support for upcoming [Framework 16 macro pad](https://frame.work/products/laptop16-diy-amd-7040?tab=modules),
keyboard emulation plugin, NodeGraph scripting system, and also some default plugins that Streamduck will ship with to
make it be able to do some things like being able to run commands and such.

With C#, hopefully I'll have working software by the end of this year, with all the plugins I wanted to make