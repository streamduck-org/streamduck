using System.Collections.Generic;
using Streamduck.Data;

namespace Streamduck.Plugins; 

/**
 * Allows to query all loaded plugins
 */
public interface IPluginQuery {
	IEnumerable<Plugin> AllPlugins();
	Plugin? SpecificPlugin(string name);
	IEnumerable<T> PluginsAssignableTo<T>() where T : class;
	
	IEnumerable<Namespaced<Driver>> AllDrivers();
	IEnumerable<Namespaced<Driver>> DriversByPlugin(string pluginName);
	Namespaced<Driver>? SpecificDriver(NamespacedName name);
	
	IEnumerable<Namespaced<PluginAction>> AllPluginActions();
	IEnumerable<Namespaced<PluginAction>> PluginActionsByPlugin(string pluginName);
	Namespaced<PluginAction>? SpecificPluginAction(NamespacedName name);
}