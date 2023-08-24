using System.Collections.Concurrent;
using System.Linq;
using System.Threading.Tasks;
using Streamduck.Plugins;
using Streamduck.Plugins.Loaders;

namespace Streamduck;

public class App {
	public ConcurrentDictionary<string, WrappedPlugin> Plugins { get; private set; } = new();

	/**
	 * Initializes Streamduck (eg. load plugins, load auto-connects)
	 */
	public void Init() {
		Plugins = new ConcurrentDictionary<string, WrappedPlugin>(
			PluginLoader.LoadFromFolder("plugins")
				.ToDictionary(p => p.Name, p => p)
		);

		foreach (var pluginName in Plugins.Keys) { }
	}


	/**
	 * Runs the Streamduck software
	 */
	public async Task Run() { }
}