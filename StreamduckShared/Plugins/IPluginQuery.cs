using System.Collections.Generic;
using Streamduck.Data;
using Streamduck.Rendering;
using Streamduck.Triggers;

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
	Namespaced<Driver>? SpecificDriver(string pluginName, string name) =>
		SpecificDriver(new NamespacedName(pluginName, name));
	
	IEnumerable<Namespaced<PluginAction>> AllActions();
	IEnumerable<Namespaced<PluginAction>> ActionsByPlugin(string pluginName);
	Namespaced<PluginAction>? SpecificAction(NamespacedName name);
	Namespaced<PluginAction>? SpecificAction(string pluginName, string name) =>
		SpecificAction(new NamespacedName(pluginName, name));
	
	IEnumerable<Namespaced<Renderer>> AllRenderers();
	IEnumerable<Namespaced<Renderer>> RenderersByPlugin(string pluginName);
	Namespaced<Renderer>? SpecificRenderer(NamespacedName name);
	Namespaced<Renderer>? SpecificRenderer(string pluginName, string name) =>
		SpecificRenderer(new NamespacedName(pluginName, name));

	Namespaced<Renderer>? DefaultRenderer();
	
	IEnumerable<Namespaced<Trigger>> AllTriggers();
	IEnumerable<Namespaced<Trigger>> TriggersByPlugin(string pluginName);
	Namespaced<Trigger>? SpecificTrigger(NamespacedName name);
	Namespaced<Trigger>? SpecificTrigger(string pluginName, string name) =>
		SpecificTrigger(new NamespacedName(pluginName, name));
}