using System.Collections.Generic;

namespace Streamduck.Plugins;

public abstract class Plugin {
	public abstract string Name { get; }

	public abstract IEnumerable<Driver> Drivers { get; }
}