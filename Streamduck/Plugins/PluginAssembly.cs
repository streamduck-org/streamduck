using System.Collections.Generic;
using Streamduck.Plugins.Loaders;

namespace Streamduck.Plugins;

/**
 * Collection of all types inside of a plugin
 */
public class PluginAssembly {
	private readonly WrappedPlugin[] _plugins;
	internal readonly PluginLoadContext Context;

	public PluginAssembly(PluginLoadContext context, WrappedPlugin[] plugins) {
		Context = context;
		_plugins = plugins;
	}

	public IEnumerable<WrappedPlugin> Plugins => _plugins;

	public void Unload() {
		Context.Unload();
	}
}