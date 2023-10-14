using System.Collections.Generic;
using Streamduck.Data;

namespace Streamduck.Plugins; 

/**
 * Allows to query all loaded plugins
 */
public interface IPluginQuery {
	IEnumerable<Plugin> AllPlugins();
	Plugin? SpecificPlugin(string name);
	
	IEnumerable<Namespaced<Driver>> AllDrivers();
	IEnumerable<Namespaced<Driver>> DriversByPlugin(string pluginName);
	Namespaced<Driver>? SpecificDriver(NamespacedName name);

	IEnumerable<Namespaced<PluginAction>> AllPluginActions();
	IEnumerable<Namespaced<PluginAction>> PluginActionsByPlugin(string pluginName);
	Namespaced<PluginAction>? SpecificPluginAction(NamespacedName name);
	
	IEnumerable<Namespaced<PluginFunction>> AllPluginFunctions();
	IEnumerable<Namespaced<PluginFunction>> PluginFunctionsByPlugin(string pluginName);
	Namespaced<PluginFunction>? SpecificPluginFunctions(NamespacedName name);
	
	IEnumerable<Namespaced<AsyncPluginAction>> AllAsyncPluginActions();
	IEnumerable<Namespaced<AsyncPluginAction>> AsyncPluginActionsByPlugin(string pluginName);
	Namespaced<AsyncPluginAction>? SpecificAsyncPluginAction(NamespacedName name);
	
	IEnumerable<Namespaced<AsyncPluginFunction>> AllAsyncPluginFunctions();
	IEnumerable<Namespaced<AsyncPluginFunction>> AsyncPluginFunctionsByPlugin(string pluginName);
	Namespaced<AsyncPluginFunction>? SpecificAsyncPluginFunction(NamespacedName name);
}