using System.Linq;
using System.Threading.Tasks;

namespace Streamduck.Plugins.Extensions; 

public static class PluginCollectionExtensions {
	public static async Task InvokePluginsLoaded(this PluginCollection collection) => 
		await Task.WhenAll(collection.AllPlugins().Select(p => p.OnPluginsLoaded(collection)));
	
	public static async Task InvokeNewPluginsLoaded(this PluginCollection collection, Plugin[] plugins) => 
		await Task.WhenAll(collection.AllPlugins().Select(p => p.OnNewPluginsLoaded(plugins, collection)));
}