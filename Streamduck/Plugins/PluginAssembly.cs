using System.Collections.Generic;
using Streamduck.Plugins.Loaders;

namespace Streamduck.Plugins;

/**
 * Collection of all types inside of a plugin
 */
public class PluginAssembly(PluginLoadContext context, WrappedPlugin[] plugins) {
	internal readonly PluginLoadContext Context = context;

	public IEnumerable<WrappedPlugin> Plugins => plugins;

	public void Unload() {
		Context.Unload();
	}
}