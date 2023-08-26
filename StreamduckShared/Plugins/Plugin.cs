using System;
using System.Collections.Generic;

namespace Streamduck.Plugins;

public abstract class Plugin : IDisposable {
	public abstract string Name { get; }

	public abstract IEnumerable<Driver> Drivers { get; }
	public abstract void Dispose();
}