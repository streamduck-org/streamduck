using System;
using System.Collections.Generic;

namespace Streamduck.Plugins;

public abstract class Plugin {
	public abstract string Name { get; }

	public virtual IEnumerable<Driver> Drivers { get; } = Array.Empty<Driver>();
}