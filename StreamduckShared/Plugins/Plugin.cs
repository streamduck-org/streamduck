using System;
using System.Collections.Generic;

namespace Streamduck.Plugins;

public abstract class Plugin {
	public abstract string Name { get; }

	public virtual IEnumerable<Driver> Drivers { get; } = Array.Empty<Driver>();
	public virtual IEnumerable<PluginAction> Actions { get; } = Array.Empty<PluginAction>();
	public virtual IEnumerable<PluginFunction> Functions { get; } = Array.Empty<PluginFunction>();
	public virtual IEnumerable<AsyncPluginAction> AsyncActions { get; } = Array.Empty<AsyncPluginAction>();
	public virtual IEnumerable<AsyncPluginFunction> AsyncFunctions { get; } = Array.Empty<AsyncPluginFunction>();
}